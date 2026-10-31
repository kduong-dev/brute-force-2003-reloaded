# Grenades (`src/bin/play_grenade.rs`)

Every grenade type comes from the loaded levels' `inventory-grenade` definitions (read into
`WeaponDef`, `src/bf/weapon.rs`); nothing per type is hard-coded except which definitions the
squad carries when several share a label (the Sentry has three, the Roller two: the ones the
recordings show, `SQUAD_GRENADES`; a level that defines only another one its squad's placements
carry, or that it puts down itself, takes that one: m14_c's and m14_d's Sentry h_f73de83d, and the
same Sentry that e16, e34 and e40 lay as live mines, see **Thermo**) and their order, which Tab
steps through and the item box shows first: the game's store order, (category, index) = the
group-type and h_e5f51266, Frag, Gas, Sonic, Energy, Light, Sentry, Roller (#281,
[play.md](play.md) "Items: the squad's starting inventory"; BF_TEST_INV's grenade types the level
defines but the squad doesn't carry are loaded too, #196). The squad carries only those seven. The
Molotov (e01's h_f42e0faa, a labelled grenade with an icon) is the mutants' weapon (#78), not the
squad's: its definition is read like the others' for an AI thrower to use. Per type:

| Field | Where | Frag |
|---|---|---|
| fuse | `timer`, counted from the first ground contact; 0 goes off on the first contact, no bounce | 1.5 s |
| explosion | h_053c429f -> a weapon: `<Damage max min damage-type radius>` | h_15889e57: 58.5-65, type 10, 8 m |
| blast effect | explosion bullet h_ec23d593 -> effect type: ALE effects, light effect, sounds (`EffectType`, `src/bf/character.rs`) | h_1fe9ae17: exp-lrg-dirt, -fire#1, -fire, -flash, -shrap, exp-fire-add; light_explosion; 145f09e5 |
| impact sound | bullet h_e5618348 | fb4b3604 |
| decal | bullet h_06a27365 -> `<decal>` | h_ff1b711e: ff534b01, RGBA 10 10 10 200, 3 x 3 |
| trail | h_18b6ab72's first slot -> effect type | h_10a5508f: grenade_trail + 10318b29 (the hiss) |
| HUD icon, label | h_e5ec3f1f, stringtable-name | fe20b919, "Frag" |
| use | h_1ee2f4ed: 3 IOU_THROW_TO_USE, 2 IOU_PLACE_ON_GROUND; function-type 8 / 13 (Sentry / Roller) | thrown |
| event sound | `<event state=7 h_fa04e025>` | e43166d1 (the meter appears) |

`BF_GRENADE_LOG=1` prints each type's fields at load (`grenade <label> ...`), each throw's launch
("Frag leaves the hand 2.13 m over the floor at the thrower's feet at 25.6 m/s, 6.0 degrees up
(mass 0.9)", #161), each grenade's first contact (how long after leaving the hand, how far from
the thrower), each blast and decal, and a throw called off.

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
  and drops a charge in progress, G then dead until it's let go (#196: [play.md](play.md), **The
  item box and the item list**; the T key that stepped the grenade types is gone, the game has
  no such button). A throw cut short (knocked down, or control handed over, before the grenade
  leaves the hand) puts the grenade back.
* **The blast**, on the ground below where it went off: the effect type's ALE effects and its
  light effect run once from that frame (`src/ale_fx.rs`; an effect without a parent now starts
  on the frame it's spawned; a "light_" effect is a point light: reach = its size, strength =
  colour x alpha x size, the tuning once fitted to the old DNA effect's light), its sounds (h_f724cb8c read as a delay: the
  Light's 0.2 s) and the impact sound. The decal is laid 0.7 s later, once the fireball has
  gone (laid at once, it showed as a hard dark disc through the added fireball), its
  width / height read as half sizes as the blood decals' are (the Frag's 3 x 3 is a 6 m quad,
  half dark over ~3.7 m; the recording's scorch is half dark over ~4.1 m), lasting 60 s and
  fading over the last 2 s (h_199870ec / h_fe4e1d82 read as life and fade, an inference), laid
  along the ground's slope under its middle. It darkens as the console's blend does: the Xbox
  blends in stored (sRGB) values, Bevy in linear light, so its alpha a is drawn as
  1 - (1 - a)^2.2 (the scorch's middle at ~0.25 of the ground, the recording's 0.2-0.3; at the
  plain alpha it was 0.47). Blasts that hurt throw loose pickups (not the Light's).
  * ALE details the blast showed up (`src/ale_fx.rs`). These three rules (and the outward
    perp quads below) are checked against the Frag recording only, so they apply only to the
    grenades' blast and trail effects (`AleAssets::load_recorded`, `Compiled::recorded`);
    every other effect (guns, hits, muzzle flashes, level fires, icons) plays as before them,
    until checked against its own captures:
    * An animated texture's frame comes from the appearance's 18657e4a over the particle's
      life: a ramp plays the flipbook over the life (exp-lrg-add's Exp5 0.61 -> 1.0: from the
      orange frames to the smoke), a constant above 0 holds that frame (exp-lrg-fire 0.70, the
      muzzle flashes 0.60), 0 plays it at its own rate (the shield icon). An inference;
      before, every flipbook played at 30 fps and wrapped, so exp-lrg-add went back to its
      bright first frames at 0.53 s: a second white flash the recording doesn't have.
    * An emitter with no rate and no initial count starts with its e7221f95 (read at 0) as a
      burst: exp-lrg-flash's 4 flares, the recording's wide white haze on the first frame
      (an inference; one particle otherwise, a gun's tracer).
    * Each effect's materials are drawn once out of sight while the map loads
      (`ale_fx::warm_up`), so their render pipelines are built before the first blast. In one
      of five captures run beside two other instances, the flash and the add layer were
      missing for the blast's first half second: the likely cause, a pipeline still being
      built. Not reproduced since (four captures, three at once, alike).
  * Added for the Energy's stun_grenade_master (grenade effects only):
    * An effect appearance (class h_0ec77ea0) makes each particle of its emitter carry the
      effect named by its h_0ec7a290 ("stun_grenade": a bolt), turned to face the way the
      particle moves along the ground (an inference). The particle lives no longer than its
      effect runs once (0.8 s, not the master's 7.49 s).
    * Those two of its pairs whose emitter isn't in the node library (stun_grenade_init's
      h_ed10c55f, stun_grenade_init#1's h_f48dc74d) get a stand-in: one particle at the middle,
      living the appearance's own lifespan (a guess, fitting the recording's flash and wash).
      Only those two, by name: exp-lrg-dirt (the Frag's, the Roller's) and exp-mine-dirt (the
      Sentry's) miss their emitters too and still draw nothing.
    * A beam whose texture is an animated one ("ARCb": a 4 x 4 sheet of arcs, 30 fps) steps
      through its frames over the ribbon's life.
    * An effect entity's scale scales its emitters' offsets (a bolt's reach is fitted to the
      body it strikes so).
  * Beam ribbons are never frustum-culled (any effect): their mesh is rebuilt every frame, and
    the bounds taken from its first few points culled the Energy's bolts as they spread.
  * The Frag's exp-fire-add is a sphere emitter of "perp" quads. Perp quads lie flat in the
    emitter's frame (the laser hits' rings show that), which drew it as a
    stack of flat discs seen nearly edge-on: a wide flat streak. From a sphere emitter whose
    particles move out at 0.3 m/s or more, a perp quad now faces out along its own direction,
    and the fireball is round from its first frame, as in the recording. Grenade effects only;
    the 0.3 m/s threshold is the demo's. The Sonic's cone emitter sonic_grenade.emt does the
    same (by name, see **Sonic grenade**). The flag read as perp (0d645074) is the game's
    GeneralApp_AngleFadeOut; how the game turns a quad is GeneralApp_FacingType (e3542300), read
    from its particle vertex shader (#73: play.md's **Missiles**, `FACING` in `src/ale_fx.rs`).
    Facing 3, on the Frag's exp-lrg-add.app and exp-lrg-shrap.rect.app among others, is a quad
    stretched along its particle's motion and turned about it to face the camera. The Therm
    Sweeper's trail and explosion and the muzzle effects are drawn so; the grenades keep the
    rules above, checked against their recordings, until they're checked with it.
  * Added for the Light's phosphor_grenade (grenade effects only): an upright streak - a
    camera-facing appearance (not perp, not motion-blurred) whose width factor (04b5fc1b)
    stays under half its height factor (0db2cc8d) all its life, with a constant Rotate - has
    no random roll, so it stands upright on the screen, and rises by its appearance
    transform's offset over its age (`UPRIGHT_WIDTH`, `Pair::rise`). Of the grenades'
    appearances only phosphor_grenade_init_spike.app is one (width 0.05 -> 0.30 of a size
    peaking at 5.9 m; offset 0 -> 2.26 m up over 0.92 s): the recording's tall thin blue beam,
    where the random roll drew a fan of long rays. The offset is read over the particle's age,
    not the effect's time (an inference: over the effect's time, repeating every 0.92 s, the
    whole beam would rise and drop back each 0.92 s; the recording's beam top flickers by ~25%
    about every 0.3 s with no slower cycle). The 0.5 threshold is the demo's. Roll 0 is upright
    on the screen, not in the world: under a steep camera pitch the streaks lie back with the
    view.
  * Also for the Light: a spark - a perp appearance born longer than 1.5 times its width
    (ASPECT at birth) on a cone emitter throwing its particles out at 0.3 m/s or more - is drawn
    as a camera-facing streak along its motion, not lying flat (`SPARK_ASPECT`).
    phosphor_grenade-shrap.rect.app (spark.tga, aspect 2.2 -> 0.25, ~100/s at 4-6 m/s within
    49 degrees of up) is the only grenade appearance it picks out: the recording's starburst
    of rays fanning up from the flare at ignition and the sparks round its base after (lg
    0397-0411, 1073-1085, 1200), which lying flat drew as short horizontal lines. The other
    grenades' perp appearances are born square or squat (aspect 0.04-1.01) and stay perp
    quads. Their gravity field stays off (#101), so the sparks fly farther than recorded (up to
    ~5 m, faint by then, where the recording's stay within ~1-2 m).
  * Also for the Light: a steady light - a "light_" effect whose emitter has a constant rate
    at which its particles overlap two deep or more - lights each particle up over its first
    1/rate s and down over its last, scaled by rate x life / (rate x life - 1), and starts each
    particle at the moment within the frame its rate says it's due (`steady_rate`): the evenly
    spaced particles' sum is then constant, at the plain particles' mean. Only light_phosphor
    is one (5.99/s, 0.514 s: ~3 at once). As plain particles, the number lit stepped between 3
    and 4 and the ground light pulsed by 3-11% about three times a second (also on main: the
    tester's measurement), where the recording's is steady within 1/255. The Frag's and the
    Sentry's light_explosion (one particle) and the Sonic's light_sonic_grenade (a keyed rate)
    are lit as before. How the console summed its lights isn't known.
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
  every item's), label and count (from 2 up), the game's widget since #196 ([play.md](play.md),
  **The item box and the item list**). The meter's fill is drawn as (226, 148,
  54) at 0.65: what the console's stored-value blend of (255, 160, 53) at 0.65 gives over the
  recording's ground (63, 63, 55), since the UI blends in linear light. The charge sound is
  queued on the frame the meter first shows (`BF_SOUND_LOG` and the capture agree). The fade's
  alpha is drawn so its strength over take01's ground (37, 27, 20) is the game's stored-value
  one (`METER_GROUND`, `METER_FULL`). When the selected type runs out, the box is blank a frame
  and then shows the first stack in the store's order, whatever its category (FUN_0014db60; the
  Sentry recording: the last Sentry set down, the box shows Frag 10; #196 take09).

* **The Gas's cloud** (`src/bin/play_gas.rs`; issue #76). The explosion h_0cc6cb8c is
  `<Damage max=97.5 min=48 damage-type=4 radius=3 h_04ea9251=5.5>`; its effect type h_065e4b95
  plays gas-grenade (the olive cloud), smoke-grenade-flsh (the flash) and
  smoke_grenade-shrap (white sparks), and its sound h_1b35643e. In the recording the health bar
  doesn't move at the blast; it drains steadily while Tex is in the cloud: 15.5 HP/s in the
  first (he walks in 2.2 s after it went off and the rate doesn't rise as he comes closer) and
  16.9 HP/s in the second (~2 m from it, starting 5 frames of 60 after the blast), stopping
  5.80 s after the first went off; the second killed him. The demo: everyone within the
  radius (3 m, the blasts' distance) loses Damage max / h_04ea9251 = 17.7 HP/s (the data's
  nearest value to the recording's; a roll of min..max would average 13.2 HP/s - a guess),
  times their damage-type factor (Flint 0.9 HP/s) and their difficulty step with STANDARD's
  squad x0.6 divided out (the recording's, **Damage** above: 17.7 on the squad on STANDARD,
  29.5 with no step (the multiplayer maps); 41.3 on an enemy on STANDARD, the demo's extrapolation: measured on the
  thrower in its own cloud, the rate holds the x0.7 too, and with that taken out it would be
  ~59; #112's), from 0.083 s after the blast for 5.5 s
  (5.58 s after it: 0.2 s shorter than recorded). The thrower is poisoned like anyone (it was
  Tex's own cloud). No blood, red tint or hurt chatter (the recording's picture only goes
  olive); one it kills dies as from any hurt. Squadmates don't leave or avoid it (not
  recorded). On the test map, instant kill kills a squadmate it reaches. A cloud keeps its own
  clock (the frame's time), so it poisons on at the same rate through a death camera and a
  hand-over.
  * The cloud's emitter (the effect's h_1c14fe91) is gas-grenade_Cone.emt#1.emt: the case-kept
    hash of its name, which the library already keys nodes by. Its puffs leave at 4-6.5 m/s
    and are held back by the effect's two air fields (gas-grenade.fld: a wind of about +-1 m/s
    swinging round over time; gas-grenade2.fld: 0.26 m/s up). `src/ale_fx.rs` now runs air
    fields, but only those checked against a recording (`AIR_FIELDS`): these two, and
    grenade_trail_rise (0.8 m/s up: the grenades' trail now rises the right way, thinner and
    more vertical than the recordings' - a partial match). The Sonic's sonic_grenade_air.fld
    was checked against its recording and stays off (see **Sonic grenade**). Every other field -
    the Frag's exp-lrg-air, every gravity field (exp-lrg-shrap,
    exp-lrg-dirt, phosphor_grenade-shrap, smoke_grenade-shrap) and turbulence field - stays off
    until checked against its own capture (#101). An appearance's fields are the effect's pairs
    appearance -> field (as Freelancer links them; FxAirField, AirField_Magnitude and
    AirField_Approach are the game's hashes of Freelancer's names). An air field pulls a
    particle's velocity toward Magnitude m/s along its +y (turned by its transform over the
    effect's time), by Approach once per 30 fps game frame - the rate fitted to the recording's
    cloud (from 12 m: per 1/60 s every puff stopped within 0.3 m and Tex stood out in front of
    the cloud; per 1/30 s it's ~6.5 m wide and ~5 m tall and veils him, the recording's ~6 x 4
    m; per 1/15 s, ~9 x 6 m). Not done: a field node's own life (h_f27fde7d; gas-grenade.fld's
    12 s) is ignored; the wind is in the world's frame while an attached particle's velocity is
    in its effect's (no attached particle has an allowed field yet).
  * The cloud lingers: still dense at +7 s where the recording's first cloud is thin by +6 s
    and gone by +7 s (scratchpad gas76/fade3.png, test76/side_tm177.png). From the data, its
    emitter runs to 4.26 s and its puffs live 2.3-4.4 s (h_0a635880's repeating keys), their
    alpha falling from 0.98 at 39% of their life: summed over the puffs, the alpha is 36% of its
    peak at +6 s and 10% at +7 s (scratchpad gas_density.py) - about the recorded timing - but
    the puffs overlap so much that the remainder still looks thick. Not brought closer: a slower
    approach (per 1/15 s) spreads the cloud wider without thinning it sooner, and reading the
    life as the curve's constant 2 s (gone by 6.3 s) would change the rule every grenade effect
    uses. The recording's camera also leaves the cloud at ~5.8 s with Tex, so its fade from
    inside isn't recorded.
  * h_19d5e391 is gas_grenade#1, an effect with no nodes (hence the load's warning), and
    smoke_grenade-shrap's appearance names no texture (the other warning): its sparks are
    drawn untextured.

What each type does so far, and doesn't:

| Type | Does | Not yet |
|---|---|---|
| Frag (#75) | everything above | the "Tech Upgrade Frag Grenade!" upgrade |
| Energy (#74) | timer 1.75, stun_grenade_master + stun_hit_s, f875b6c6, decal h_f4d65518; its bolts crawl out and carry the damage (39-97.5, 9 m) to every body in reach, throwing down the living and throwing corpses (see **Energy grenade**) | the stunned_fx 30 s arc; the per-liquid effects h_0cbcb8f2; the "Tech Upgrade" |
| Gas (#76) | timer 1, gas-grenade's olive cloud (with its air fields), 1b35643e, the poison: 17.7 HP/s within 3 m for 5.5 s, Flint x0.05 | the "Tech Upgrade" (the ticket's); the cloud's fade (lingers ~1-2 s, see above); the flash's ~1 s flicker (smoke-grenade-flsh's keyed emit count: one burst here); the shared HUD's damage feedback (white bar with a dark-red trailing segment, the direction chevron) |
| Light (#77) | timer 1.5; phosphor_grenade (a tall thin blue beam of upright streaks, the glow and sparks at its base) + light_phosphor (a blue-white point light, reach 18.8-24.3 m) burning 30 s as the data has them (the spike's rate drops to 0 at 29.45 s); ignition h_1080aaf1 and f5260e35 after 0.2 s; no damage, no decal, no tint, pickups left alone; the canister stays where it lies, its trail stopped, until the effects have run (31.1 s) | the end of the burn isn't recorded (the recording shows >= 15.1 s); the beam is paler and less solid than recorded and the base's white core smaller (see **Light grenade**); the sparks fly too far (no gravity field, #101); the light's strength not measured against the recording |
| Sonic (#81) | goes off on first contact (timer 0), grenade_sonic (dome, ring, godrays) + light_sonic_grenade, f36fb063, decal h_fb24bcb7; the damage comes with the ring, the thrower's falls off by 5 m, nobody is knocked down (see **Sonic grenade**) | the "Tech Upgrade"; the godrays (broad columns, not two narrow shafts), the ring (a wall, not a flat band), the decal's look |
| Roller (#79) | set down (place_hi); seeks the nearest hostile in sight within 50 m every 0.5 s (from 1 s), hops after its point (a fit to the takes: 3.1, 5.6, 7.2, 8.5 m/s at its first hops, a ceiling of 8.5), stops dead at walls, goes off within 2.5 m of its point or at its 25 s fuse; h_013b3ca8 (the Frag's six effects + light_explosion) + h_065168c9; its rolling sound (see **Roller**) | the per-material ground puff h_0cbcb8f2 (no grenade has one yet); a shot or a blast taking its 1 hp; the set-down hand (above) |
| Thermo (#82) | set down (place_hi), 15 s fuse from the landing; m09_b's own charge set down armed at the load, its fuse held by the startup cinematic (24 s after the level's start); the Roller's effect set at 3 m + the impact sound h_16611581 (see **Thermo**) | whether a shot sets it off; enemy damage by distance; its look and sounds (not filmed) |
| Sentry (#80) | set down (place_hi), lies there with its red tips lit (#300); goes off for a hostile within 3 m with no friend within 3 m, or when shot (see **Sentry**), or by a blast that takes its 1 hp (see **Interactive scenery**); exp-mine + light_explosion, h_145f09e5 | disarming an enemy's mine; the AI keeping clear of it; the tips' glare; the thrower's damage beyond the radius (below) |

Not done for any: ALE fields other than the Gas cloud's and the trail's rise (exp-lrg-air,
the gravity and turbulence fields: #101); decals don't
follow uneven ground (a plane along the slope under the middle: a big scorch on a bumpy
hillside is partly buried); the sounds' play-length; distance falloff for blast sounds beyond
a volume.

## Sentry (`src/bin/play_sentry.rs`)

The proximity mine (h_e5f1f063, function-type 8 IFSET_PROXIMITY_EXPLOSIVE): set down as the
Roller is, its `timer` 9999 s never runs out; what sets it off is the item set's handlers in
default.xbe (vtable 0x39bc50; the range test FUN_00146cb0, the check FUN_00146e00 and the
on-placed FUN_00147570 have Ghidra's C; the per-frame 0x147620 and the target filter 0x147430
have none and were read in the disassembly):
* Once it's down (on-placed 0x147570; there is no arming delay in the code), it checks every
  0.05-0.15 s (0x147620: a timer passing 0.15 s, restarted at a random 0-0.1 s); the first
  check comes 0.15 s after it's down. A check that says so sets it off at once.
* The check (0x146e00) goes through every living character. A friend - on the mine's team
  (its owner's when it was set down, kept at +0x1c0 by 0x147570), or the thrower - within the
  radius ends it: no blast this time. Anyone else within it is a target: characters on other
  teams always, and on the mine's own team only for a team the team table (0x3ffd80) makes
  hostile to itself (its diagonal: team 7 only). It goes off with a target and no friend within the radius. The radius is the item's
  h_0a811e94 (3 on every Sentry; read into `WeaponDef::proximity_radius`), a 3D sphere round
  the mine (0x146cb0).
* A character's point for it is 0.9 m above the feet: a fit to the recording's enemy brought
  up a slope (set it off with its feet 2.51 m across and 0.63 m above the mine, not at
  2.61 / 0.67; the 3 m sphere puts the point 0.81-1.01 m up). On flat ground that's 2.88 m
  across. The mine's point is its model's middle (the model is 0.14 m tall).
* Characters whose type handles mines and that know of this one count only standing on it:
  within 0.3 m across and 1.5 m up or down (0x147430). A type handles mines when its
  `<combat-skill>`'s hazard list (`<h_0c437951 action>`, the type's bytes +0x294 the filter reads
  at +0x294 + 3) holds 3, the hazard kind a Sentry gives itself (0x147690): Tex's alone
  (`SkillData::hazards`). It knows of every mine but his own squad's: his skill's +0x94 (Tex's
  Berserker, 0x13d8f0) says no for a mine whose owner is in his squad, every other skill's
  (0x280040) yes. So Tex steps round the level's mines (#228) and counts as anyone for the
  squad's own; the test hook's `wise` hostile has the rule too.
* It has 1 hitpoint: a shot (anyone's) whose line passes through its model's bounding sphere
  stops there and sets it off when it arrives. The shots are tested against the mines as they're
  made (play.rs's `update_player`, before friendly fire): the first thing on the line takes the
  shot - a teammate in front of the mine is hit and the mine isn't; a teammate behind it isn't
  hit. Bodies on the ground don't stop shots in the demo (they're shoved and the shot goes on),
  so a mine behind one is still struck, and bodies beyond the mine aren't shoved. The loose
  pickups (play_pickups.rs, after `update_player`) see the shortened shots.
* The squad is one team, so the squad's mines never go off for the squad (as recorded: Tex
  walking or running over his own, standing beside it, Flint stepping onto it at 0.16 m). A
  test hook's hostile, the test map's enemy NPCs ([testtools.md](testtools.md)), the levels'
  terrorists, or a shot, sets one off. Nobody can pick one back up.
* **The level's own mines** (#228): an item object of a Sentry type placed with h_1cb7d5f2
  (`Placement::placed`; the tutorial's three `<h_1d1d4f05>` item spawns in the mine field, at
  (92.6, -71.8), (95.3, -82.3), (98.2, -75.8)) is set down and armed as the level loads (its
  attribute handler 0x14a0a0: +0x1bc bit 1, the item set's +0x10), on its `item-alignment`'s
  team (+0x1c0: 1, the enemies') and owned by nobody; the level's own model of it (the one of its
  name, or for an unnamed one the nearest of its type within 1 m) is hidden and the mine drawn
  in its place. It goes off for the squad's Confed marines within its 3 m, but
  Tex only sets it off standing on it (above): take09 has him 0.84 m from the first for 8 s
  (71-79 s) before the disarm. Going off it sends DESTROYED to the triggers that listen to it
  (0x149520 at 0x1496fc): the tutorial's routers 446 and 470, and calculator471, which counts the
  three to end the mine lesson. The same rule arms e16's, e34's and e40's Sentry mines
  (h_f73de83d: 14, 6 and 7) now that `load_kits` makes their kit (#82, **Thermo**); before, the
  demo had no kit for them there and left them out. Not checked against those levels' footage.
* **Its look: red tips, whoever set it down (#300).** The mine's model h_075885e5 (every Sentry
  set down: the squad's h_e5f1f063, the levels' h_f73de83d, the AI's) has a lights geoset on
  its four tips: the wrapper h_0a08f6b5 (stencil 50) round the self-lit material h_f9761b6c,
  untextured, h_e01baa40 0.757 0 0, `blend-mode` 2, `alpha` 1, no scroll. The self-lit shader's
  setup (FUN_0008fd20) binds the default white texture for none and draws it opaque, lit by
  nothing: saturated red, steady (the tutorial's takes 20 and 21: the tips measure (167, 18, 26)
  on average through the video's blur, up to (199, 41, 35); #80's t80_07 / t80_08, Tex's own on
  m03_a: the same). Nothing colours a mine by its team: its on-placed (FUN_00147570) and its
  updates write no effect, light or material, and no code tints a material at run time
  (code-map.md, FUN_00201710). What does differ by who set it down is the HUD: the use ring (blue
  on an enemy's, the disarm; red on your own, refused: FUN_00121860) and the crosshair's lock
  (red on an enemy's only: FUN_000bf6d0), #145's. The blue Sentry the product owner saw is its
  pickup form h_ecfe0d8c (model h_095daa18, lights h_031724e6 -> h_0ed4a3dd 0.098 0.342 0.829),
  which only sdm_e34 (five) and mp8 (two) lay, as spawn-trigger templates: the demo copies no
  grenade pickup yet (pickups.md, "Pickups from spawn-triggers"), and the level viewer draws them
  as it draws placed glow shells, blue and see-through.
  * The demo drew every untextured material of a model plain grey (`scene::material`); now an
    untextured self-lit one is its colour, unlit, blended as FUN_0008fd20 says
    (`level_scene::self_lit`). A scan of the 53 levels' object types and character types: the
    Sentry's tips are the only such parts of the grenades, weapons, missiles and characters;
    two debris pieces (h_fc5e4d5f and h_fd5c98c6, on 9 and 8 levels) carry a dark red one
    (h_0049b1eb, 0.529 0.024 0.008, opaque), drawn so now when a broken object throws them; the
    medkits' crosses are flat glows that play_pickups.rs and level_scene draw their own way, and
    the placed objects' self-lit parts are level_scene's (unchanged: levels.md).
  * Not done: the tips' glare. Their wrapper's stencil 50 is a glare key (#289: the bright pass
    keeps the red's luminance, 0.226, over 50/255), but a bevy standard material writes 1 to the
    picture's alpha; a keyed material would have to follow Hawk's fade and the Vengar view,
    which swap standard materials. The takes show the halo faint at this size.
  * Checked (`BF_MAP=tutorial BF_NO_SQUAD=1 BF_TEST_GOTO=92.6,-68.6,92.6,-80 BF_TEST_GO_AT=100
    BF_CAMERA_PITCH=-0.35`, frame 24: the level's mines red-tipped where main's were grey; and
    Tex's own on m03_a, `BF_MAP=m03_a BF_NO_SQUAD=1 BF_TEST_GOTO=-185.7,-39.1,-185.7,-30
    BF_TEST_GO_AT=100 BF_TEST_GRENADE_TYPE=Sentry BF_TEST_THROW=1.0,0.1 BF_CAMERA_PITCH=-0.4`,
    frame 43: at his left foot with four red tips, as t80_08 shows it).

Its blast is the shared one (h_01f142eb: 62.5-88.5, type 10, radius 4; effect type h_0b69c2f2:
exp-mine + light_explosion, 145f09e5; decal h_f5ccedb0). On sdm_e34 the squad's Sentry is still
h_e5f1f063 (e34 defines it as well as h_f73de83d, whose explosion is the Energy's h_f7d6b42e:
the squad carries h_e5f1f063 wherever it's defined, `SQUAD_GRENADES`). Not matched: the
recording's Tex took 16.5 and 14.8 HP from blasts 6.0 m away, outside the data's 4 m radius;
the demo's thrower is hurt only within the radius (0.2 x 62.5-88.5 = 12.5-17.7 HP), so at 6 m
he takes nothing. Traced so far, not implemented (it would change every grenade's blast, its
own ticket; each function is in [code-map.md](code-map.md)): 0x149520 queues the explosion on
the world, whose update (FUN_000d9a20) makes it a blast object (FUN_0022bfc0) at the grenade:
damage +0x20 a roll of min..max (times the queued factor, 1), radius +0x24, life +0x28 /
+0x30 the Damage's h_04ea9251 (0 here: it deals once), a delay of 0.05-0.25 s (+0x2c), and no
speed: it doesn't move. The world turns one queued explosion into a blast per update. The blast
pool's tick (FUN_00225220) waits out the delay; FUN_00224a90 gathers the bodies within the
radius of that fixed point (characters need a clear ray to it); FUN_00224770 deals each one
through the explosion weapon's damage handler, where FUN_0022c4d0 applies the Damage's
distance rule: h_011cb154 false (the Sentry's) half the roll anywhere inside the radius; true,
the whole roll within half the radius, then (1 - 2 (d / r - 0.5))^2 of it to the edge. The age
taper (FUN_00225060 -> FUN_00223780) needs flag 0x40, which nothing sets, and FUN_00225440
flies the bullet-type 0 projectiles that share the pool, not blasts. So the code
read doesn't explain Tex's hits at 6 m. The demo keeps its shared rule (Damage max falling to
nothing at the radius; the thrower 0.2 x a roll of min..max inside it).

`BF_SENTRY_LOG=1` prints each check with a target within radius + 1.5 m (3D and across
distances to the nearest target and friend, and the result), each shot that strikes a mine and
each mine set off. Test hooks (with `BF_TEST_GOTO`):
* `BF_TEST_HOSTILE=<character>[,<from m>,<to m>,<m/s>][,wise]`: that squadmate (brutus, flint,
  hawk, tex or 0-3) is on another team and stands still out of the squad AI. Once the first
  Sentry is down it's put `from` m (5) beyond it, on the far side from the thrower, waits 1 s,
  then is stepped in toward it at `m/s` (0.25) down to `to` m (0) across; held where it was
  when the mine goes. `wise`: it handles mines and knows this one.
* `BF_TEST_MINE_FRIEND=<character>,<m>[,<s>]`: once the first mine is down, that squadmate
  is held `m` m beside it for `s` s (default for good), then let go to the squad AI; until
  then it follows the squad as usual.
* `BF_TEST_SHOOT_MINE=<s>`: `s` s after the first mine is down the player turns the crosshair
  onto it and fires from the hip.

Verified (test map, `--test BF_TEST_INSTANT_KILL=0 BF_TEST_GRENADE_TYPE=Sentry
BF_SENTRY_LOG=1 BF_COMBAT_LOG=1`, Tex backing away from it with `BF_TEST_GOTO=20,20,20,10
BF_TEST_STRAFE=0,-1 BF_TEST_THROW=1.05,0.1` unless said):
* Trigger distance (`BF_TEST_HOSTILE=flint,4,0,0.5`): stays at 3.05 m (2.93 across), goes off
  at the next check, 2.99 m (2.87 across), with Tex 5.3 m away; Flint, 3.0 m from the blast,
  takes 22 HP and is knocked down. The blast frames show exp-mine's fireball as before.
* A friend near (`BF_TEST_HOSTILE=flint,4,2,1 BF_TEST_MINE_FRIEND=hawk,2,6`): Flint stands at
  2.0 m from +3.2 s, the mine stays while Hawk is at 2.2 m, and goes off at the first check
  with Hawk past 3 m as he walks off (3.02 m; an earlier run, 2.95 m: stays, 4.19 m: goes off).
* A teammate behind the mine on the shot's line (`BF_TEST_SHOOT_MINE=1.5
  BF_TEST_HOSTILE=hawk,4,4,0`: Hawk held 4 m beyond it): the shot strikes the mine 6.5 m out
  and Hawk isn't hit. (A teammate in front of a mine wasn't staged.)
* The thrower near (`BF_TEST_GOTO=20,20,20,20 BF_TEST_THROW=1.5,0.1
  BF_TEST_HOSTILE=flint,4,0.5,1`): Tex at 0.58 m across, Flint walks in to 0.5 m and stands
  there 4 s: it never goes off.
* `wise` (`BF_TEST_HOSTILE=flint,2,0,0.5,wise`): stays at 0.47 m across, goes off at 0.20 m.
* Not matched or untested:
  * The blast's first ~0.2 s is grey-white in the demo where take10's is orange from the start
    (the shared exp-mine rendering, `src/ale_fx.rs`).
  * A mine set down on the move slides ~0.46 m after it lands (the placed grenade keeps the
    hand's speed; no footage of placing on the move; untested against the game).
  * Placing isn't possible crouched (`BF_TEST_CROUCH` with `BF_TEST_THROW` places nothing): the
    shared `can_throw` (the crouch is an action), unchanged from main; whether the game lets
    one place crouched isn't recorded.
* Shot (`BF_TEST_SHOOT_MINE=0.7`): struck 2.3 m out, goes off the same frame; Tex 3.1 m from it
  takes 15.7 HP. From 10 m (`BF_TEST_SHOOT_MINE=2.2`, a laser bolt): goes off 0.07 s after the
  shot; Tex takes nothing (outside the radius, where the recording's Tex took ~15, above). Since
  #136 the bolt strikes it on its first sweep that meets it (play_bolts.rs): the LZR-23's from
  9.6 m, 0.1 s after the shot.
* The level's mines (#228, `BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_SENTRY_LOG=1
  "BF_TEST_PANEL=a,1;a,1;a,1;a,1"`): the three armed at the start, team 1. Tex walked up to the
  first (`BF_TEST_GOTO=92.6,-62,92.6,-71.0`, after the mine lesson's cutscene) stops 1.08 m
  across from it and it stays through every check; stood on it
  (`BF_TEST_GOTO=92.75,-71.6,92.75,-71.6`) it goes off at the first check, 0.22 m across:
  DESTROYED -> routers 446 and 470 -> calculator471 at 1 of 3; Tex takes 39.7 (115 -> 75.3).
* Regression after #228 (the hazard rule): the trigger-distance and `wise` runs above give what
  main's build gives (stays at 3.05 m, off at 2.99 m, Flint taking 31.0 in both: the roll's,
  where this page's first run had 22; `wise` off at 0.20 m across).

## Roller (`src/bin/play_roller.rs`)

"Round and intelligent - the rolling bomb seeks enemies when deployed." The squad's h_fba7c475
(m09_b's four pickups are the twin h_15f9c51a): set down as the Sentry is, its `timer` 25 s, its
explosion h_f213e5b1 (Damage 117-156, type 10, radius 5, core; effect h_013b3ca8: the Frag's six
ALE effects, light_explosion and h_065168c9; the Frag's scorch). What it does between is its item
set's, IFSET_ROLLING_BOMB (function-type 13, vtable 0x39ba60; every function is in
[code-map.md](code-map.md); the leads and the takes are `todo/79-roller-grenade/`):
* **The let-go** (0x148640 -> FUN_00148720) makes its seek helper: a point h_11884f2e (50) m along
  the placer's horizontal aim (0x10fbf0; the demo's aim frame, the camera's turn) at the hand's
  height, the first check due in 1.0 s, no target, the placer's team (+0x1c0) as the bomb's.
  Measured live in the takes: the point 50.00 m along the aim, at the hand's height.
* **Every frame** once it's out of the hand (0x148690 -> FUN_001481c0, FUN_00148220): the check
  timer runs down; a target out of sight has 5 s before it's dropped. **A check every 0.5 s**
  (the first 1.0 s after the let-go): with no living target, or every 10th check (h_0a811e94), a
  **target search** (FUN_001483e0): the nearest living character of a team the team table makes
  hostile to the bomb's (0x3ffd80: never the squad for the squad's), not the one who set it down,
  whose hips (the character's node, `missile::root_point`: the feet + its clip's root-height,
  1.12 m on the Shrike Heavy; the takes' point is the enemy's record + 1.1 m) are within 50 m (3D)
  and in sight; a held target is only replaced by one within half its distance. No cone. Then the
  target in sight moves the point to its hips; out of sight, the point stays.
* **The sight line** (FUN_00148100: the standard line-of-sight filter 0x39cc18, skipping the ball
  and the target) meets the terrain and the placed objects' collision only: not the level's
  blockers or hinters, characters, corpses, items, grenades, effects or liquids (`clear`: the
  arena's `ray_shot`, which leaves the blockers out; on the flat test floor its ground, pillars
  and pieces), from the ball's middle to the hips. Cast from the takes' logged positions (the
  ball's node and the enemy's hips at each of the 116 checks of takes 05-18, `BF_ROLLER_RAYS`),
  the demo's ray agrees with every one: takes 05 (h_09a6856d), 08 (h_fc0ef12e, h_e04e5a0e,
  h_09a6856d, h_04e79fb8, h_fbdcd828), 11 and 12 (h_04e79fb8, the 10 m object near the enemy:
  not the terrain, the leads' open question) and the gate panel of 17 / 18 (h_0792d16a) block
  the misses, every find is clear, and take12's one out-of-sight check meets h_f455bbb6.
* **It goes off** when its point is within 2.5 m across (half the explosion's radius) and between
  1.5 m below and 2 m above the ball (exclusive): the fuse is set to 0 and armed (+0x1d4, +0x1dc),
  so the blast is this frame's, through the fuse's cutscene gate. The rule is on the point, not
  on anyone: an enemy unseen behind a wall 1.7-2.1 m away doesn't set it off (take18). With no
  target it rolls on and goes off at its 25 s fuse, counted from its first contact.
* **How it rolls** is Ipion's: the game spins the ball toward the point every frame with the same
  angular push (FUN_002bad80 with 4.0, about the level axis square to the way to the point) and
  the ground's friction turns the spin into the roll, none of it in the game's code. The demo's is
  **a fit** to the takes' 20 Hz logs of the ball (`DRIVE_PUSH` and the constants after it). The
  logs show the ball **hopping**: between ground contacts every ~0.6 s (0.4-0.85 s, 0.25-0.8 m
  high) it flies straight (its speed along the ground holds, gravity 9.8), and only a contact
  changes its way. So the demo keeps a drive velocity for the spin, growing toward the point at
  11.5 m/s^2 (times the horizontal part of the unit vector to it, as the push axis is) and held
  back at 1.35 /s along the way to the point (a ceiling of 8.5 m/s) and 1.6 /s across it; each
  contact takes the ball's velocity along the ground (in the ground's plane) 0.8 of the way to it,
  and it leaves again at 2.94 m/s along the ground's normal. A ball that isn't coming down into the
  ground (the ground rose under it) is only set on it, so slopes slow it. It turns with its drive
  (the drive's speed over its radius, 0.19 m: its archetype's sphere, which it rests and meets
  walls by).
* **Walls** stop it dead (no bounce): its move along the ground is swept with a ray first (a thin
  gate leaf let the push-out send it through), and it stays against the wall, pressed toward
  its point (takes 17 / 18).
* **The level's own Rollers** are left out for now, pending #316: m09_a sets one down live as it
  loads (h_0bde6982, item-alignment 1), which the item handler 0x14a0a0 lets go with no placer (its
  point 50 m back along its own frame's third column), but its side is in doubt: FUN_00148720
  copies the item's team into the helper once, at the let-go, and if 0x14a0a0 lets it go before
  `item-alignment` is applied, the helper's team stays -1 and it never seeks. So `place_mines`
  skips it and the level's own model of it stands as before. The other levels' (e16, e23, e34,
  m05_b, m09_x, m14_a, m14_c, m14_d) are spawn-trigger templates, out of the world until a copy
  is made, which the demo's spawn copies don't do for items.

`BF_ROLLER_LOG=1` prints the let-go (the point and its heading in the logs' atan2(dz, dx)), every
check (the search's candidates with their distance and what blocks their sight line, the target
and the point), each ground contact (the speed along the ground before and after, the drive) and
the proximity blast; `=2` also the ball, its drive and its point every frame.
`BF_ROLLER_RAYS=<file>` casts the sight line once the map is up for each line
`x1 y1 z1 x2 y2 z2 [label]` of the file (`#` comments) and prints what it meets first, how far
along and where, with a placed object's index and type.

Verified on e34 against the takes, each staged from its log (`-- --test BF_MAP=e34 BF_NO_SQUAD=1
BF_TEST_GOTO=-49.49,25.25,48.64,6.05 BF_TEST_GO_AT=100 BF_TEST_GRENADE_TYPE=Roller
BF_TEST_THROW=1.0,0.1 BF_TEST_INSTANT_KILL=0 BF_ROLLER_LOG=2 BF_CAPTURE=<dir> BF_CAPTURE_FPS=30`,
the enemy as `BF_TEST_NPC=0.3,h_1c5780f8,enemy,frozen,<x>,<z>,<y>`: the take's A50 Heavy, the
Soldier h_e9a41520; the gate from `BF_TEST_GOTO=-55.20,35.28,-56.40,135.28`): the let-go's point
lies at -11.1 degrees (the takes' -11.1). Straight (take09's staging, no enemy): the hops leave at
2.97, 5.72, 7.25, 7.91 m/s (the take: 2.95, 6.08, 7.23, 7.93), the 0.5 s windows from the let-go
0.4, 3.0, 5.6, 7.0, 7.7, 8.1, 8.3 m/s (the spec: 0.5, 3.1, 5.6, 7.0, 7.5, 8.4), the first check
0.97 s after the let-go; it climbs the rock ~38 m out and wanders on it, as the take's did (the
demo's higher up), and goes off 25.03 s after its first contact. The blasts (the let-go to the
blast; the take's in brackets): take10 1.1 s (0.94), take11 2.7 (2.97: unseen through h_04e79fb8
until 3.1 m, as the take), take12 5.67 (6.05: found at the third check, 31.5 m, the take's 30.9;
lost once), take14 2.17 (2.38; Flint never a target), take15 2.77 on A, the nearer (3.16), take16
2.90 on B, the nearer (3.78), take06 3.73 (4.07), take07 4.07 (4.48); takes 08 and 13 never find
one. The wide turns: take06's 109 degrees, the way first off by 20 degrees 0.6 s after the find
(0.7), within 20 degrees of the bearing 1.8 s after (1.2), slowing to 3.5 m/s (0.9); take07's
132, 0.6 s (0.5), 1.3 s (1.0), 2.4 m/s (2.5). Takes 17 / 18: stopped at the panel at z 41.13
(the take's 41.1) and stayed, no target, no blast (path within 0.9 / 1.7 m rms of the take's).
take10's frames (`BF_CAPTURE_EVERY=2`, frames 30-94) beside the take's (2.0-4.6 s): the same
view, the ball let go and hopping off toward the enemy, then a white flash, the red damage tint
and the yellow-white fireball; Tex takes 50 HP in the take where the demo's thrower share gives
less (**Damage**: #112's).
* Not matched or untested:
  * take05's 42-degree target: the take's first check missed it (h_09a6856d) and the ball
    circled it for 4 s (the blast at 5.9 s); the demo's ball, a little to the left of the take's
    at that check, sees it at once and goes off at 2.77 s. In the fit's own simulation, fed the
    take's logged points, the ball circles it for 2.5 s and goes off at 5.3 s, but the circle is
    a knife edge of the fit (1.6 /s across circles it, 1.65 doesn't).
  * The game's ball turns its first hops 5-13 degrees off the aim (the takes scatter -21 to +2
    degrees round the -11 point); the demo's go straight at the point. The demo's hop is always
    2.94 m/s (the takes' 1.9-4.2); a wide turn slows it less (3.5 m/s against take06's 0.9).
  * On the rock (take09) the take's ball rolled back from y 33.8; the demo's climbs to 37 and
    wanders there.
  * At the gate (take17) both keep hopping against the panel, but the take's shuffles 3 m along
    it (x -54.2 to -57.1) and comes off it up to 1 m; the demo's hops in place, its drive straight
    into the wall (frames 90-270 of the staging with `BF_VIEW_YAW=0.7`).
  * The set-down hand and point (above, **Placing**). A Roller shot or caught in a blast (1 hp),
    a target on a ledge or in a pit, a walking target, the re-search at the 10th check, the
    half-distance switch and the 5 s drop (code only: no take reaches them), a squadmate in the
    blast, the fall-off between 2.7 and 5 m: not filmed.
  * The per-material ground puff of its explosion (h_0cbcb8f2: dirt h_e919bf9e ...): no grenade's
    is drawn yet.
  * Its blast's damage to enemies (the tester's runs of the stagings): the A50 Heavy survives a
    Roller going off 2.4-2.6 m from it with 10-22 HP, where 8 of the 9 takes' blasts killed it. The
    shared rule (`hurt_by_blast`: Damage max falling to nothing at the radius) gives about half
    there (156 x 0.5 x 1.4 = 109); the game's (FUN_0022c4d0) gives a core blast's whole roll within
    half of r', the radius less the victim's size (117-156 x 1.4 = 164-218): #112's.
  * The sight line starts at the ball's middle, its radius (0.19 m) over the floor at rest, where
    the game's starts at its node, ~0.3 m over the floor (the spec's §4: the logged node 29.3-29.5 on
    ground at 28.9-29.1). A ray grazing a bump is a little likelier to be blocked in the demo.
  * The game's sight ray is also met by fly-blockers (its kind mask 0x58 keeps PID_FLY_BLOCKER,
    kind 8); the demo's arena has none.
  * The wall sweep (play_grenade.rs `fly_grenades`) casts once along the frame's move and keeps the
    first face it meets: when that face is floor-like (a slope under 65 degrees ahead), the frame's
    sweep stops nothing, and a wall just beyond it is caught by the next frame's sweep.

## Light grenade

The Light (h_fd1a966d, only in m09_a/b/c/x) is a flare: thrown as the others, it goes off
1.5 s after it lands with its explosion h_ebdfb28e (Damage 0-0, radius 1, no decal), whose
effect type h_1d09922e is phosphor_grenade and light_phosphor, with the impact sound
h_1080aaf1 at once and f5260e35 0.2 s later.
* phosphor_grenade: init_spike (3 at once, then ~30/s until 28.8 s, none from 29.45 s; each
  streak lives 0.6-1.3 s, white turning blue, 0.2 m growing to 5.9 m tall at 58% of its life
  and 0.3 m wide, rising 2.26 m) makes the tall thin blue beam, drawn as upright streaks (see
  the ALE rules above); init_glow (small-flare.tga - a lens flare with rays - at 60/s for
  0.068 s of every 0.103 s, ~40/s all through the burn, each 0.545 s, white turning blue,
  alpha <= 0.4, thrown up at 1.5-3.7 m/s) the white glow round the base, keeping the random
  roll; shrap (~100/s all through the burn) the rays fanning up at ignition and the sparks
  after, drawn as sparks (see the ALE rules above).
* light_phosphor: a point light (~6/s, each 0.51 s), (0.655, 0.725, 0.847) -> (0.333, 0.620,
  0.776), reach 18.8 -> 24.3 m, lighting the ground and the characters round it; steady (see
  the steady light above).
* The canister stays where it lies, its trail smoke stopped, until the effects have run (31.1 s:
  the 30 s emitters and their last particles), then goes (`play_grenade.rs` `stays`: a grenade
  whose explosion does no damage; the recording's flares lie under their beams, and the data
  has no flag for it - an inference). `BF_GRENADE_LOG` prints "canister gone" when it goes.

Verified (captures in the scratchpad's light/):
* Test map, close (`--test BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.1 BF_CAMERA_PITCH=-0.35
  BF_CAMERA_DISTANCE=4 BF_VIEW_YAW=0.3 BF_TEST_GRENADE_TYPE=Light BF_GRENADE_LOG=1`): down
  2.40 s, ignition 3.93 s ("effects run 31.1 s"); a starburst of rays fanning up on the
  first frames, then a tall thin beam of upright blue streaks from +0.27 s with a white core
  and sparks at its base, Tex's arm and the floor lit blue-white, the canister lying in the
  glow. Side by side with the recording's third ignition (lg f1070, by time since it): the
  starburst at +0.07-0.27 s, the beam's height against a character and its flicker match; the
  recording's also shows two or three parallel streaks at times (f1130). Before, the same
  capture showed a squat glow with long rays fanned in every direction, and no starburst.
* The ground light: the mean of a patch of floor near the flare over 30 frames (+0.8 to +2.7
  s) varied by 2.7% before, dipping one frame in five; now by 0.6%, with no dip.
* Test map, the whole burn at 5 fps (`BF_CAPTURE_FPS=5`, throw hold 0.2: at 5 fps a 0.1 s hold
  falls between steps): blast at t 4.20, the beam and light gone by +30.2 s; "t 35.40: Light
  canister gone" (+31.2 s, the 31.1 s rounded up to a 0.2 s step), and the canister is in the
  frames up to t 35.2 and gone at t 35.4. A capture's frame n shows the time (n + 1) / fps (the
  blast logged at t 4.20 is first seen in frame 20 at 5 fps), so frame numbers read as n / fps
  come out one step early.
* m09_a (`BF_MAP=m09_a BF_TEST_GOTO=28,-118,28,-118 BF_VIEW_YAW=0.3 BF_TEST_THROW=3,0.3`; the
  level has no start points, so the test start is needed): the flare lands ~12 m off in a pool
  and the beam stands against the dark rock. The demo's m09_a is lit much brighter than the
  recording's, so the flare's light is far less striking than in the dark recording.
* Frag and Sonic, the same capture on main's build and this one: the frames differ no more
  than two runs of one build do (Energy likewise at the first change; the later rules pick out
  none of its appearances).

Not matching:
* The beam is paler and whiter than the recording's saturated blue (its middle measured
  (97,145,174) on black against the recording's (48,107,176) to (93,196,255)), and looks less
  solid: the recording's is one dense column, white at the bottom. The demo adds sprites taking
  their colour as linear light (play_fx.rs: right on a mid-grey picture); the console added
  stored values, where a few overlapping dark-blue streaks already reach a solid bright blue,
  and more of them white. That rule is shared by every added sprite and is left to its own
  ticket.
* The white core at the base is smaller than the recording's (about a character's shoulder
  width there), likely the same blend rule: it's the glow's overlapping flares.

## Sonic grenade (`src/bin/play_sonic.rs`)

"Triggers a powerful sonic blast upon impact." Timer 0: off at its first contact, no bounce;
the explosion h_faced66c (Damage 39-71.5, damage-type 7 = DTYPE_SONIC, radius 10); the effect
type h_0c18f6cc: grenade_sonic (sonic_grenade: the dome and ring; sonic_grenade_flash: godray
shafts) with light_sonic_grenade and the sound f36fb063 at the first blast frame; decal
h_fb24bcb7; its trail grenade_trail with f6dbdb17. Measured from the recording
(todo/reference/Sonic Grenade.mp4, the reference and tester agents' frames; the second blast at frame
449, camera still): a glowing dome growing from +0.03 s, flattening into a ground ring at
~0.25-0.3 s, godray shafts +0.27-0.85 s, the glow gone by ~1 s; hits at +0.17 s (~2 m),
+0.23 s and +0.47 s; no camera shake.

* **The damage comes with the ring** (an inference from those times): every living body within
  the 10 m when it goes off is hurt once, d / 11.68 m/s after it (sonic_grenade.emt's speed at
  its first key), no sooner than the other blasts' 0.1 s: +0.17 s at 2 m, +0.43 s at 5 m,
  +0.86 s at 10 m (on the first frame at or after that time). A body out of the radius by then
  is missed. Damage max falling to nothing at the radius (the other blasts' rule), times the
  damage-type factor.
* **The thrower** takes 0.653 x Damage max x (1 - d / 5 m): 28 HP at 2 m, nothing from 5 m
  (on STANDARD: measured there, it's taken through the difficulty step like the Frag's, see
  **Damage**). Fitted to the recording's two throws, not the game's formula: Tex lost ~28 HP at ~2 m (T1,
  below the data's minimum of 39) and nothing at ~5 m (T2: his health frame flashed red, the
  bar didn't move, no tint; with ~9.5 HP left a flat share would have killed him). A thrower at
  1, 3 and 4 m (a new shot) would settle the shape. The red tint comes only with a hit of
  1 HP or more on the player (`TINT_MIN`, a guess: the falloff still gives a few tenths of an
  HP at 5.0 m, measured from the pelvis, where T2 had no tint); it isn't scaled with the
  damage, since only the Frag's ~12.5 HP tint was measured.
  The tint quad keeps the picture's alpha (`KeepAlpha`, play_grenade.rs), which carries the
  glare's key (#289, levels.md § The glare). The game lays the tint over the finished picture,
  after the glare; the demo multiplies it in before, so while it's on the glare dims with it.
* **Nobody is knocked down by it** (the recording's Tex and an enemy ~1 m away stayed up; a
  hit of 15 HP or more floors a body two times in three otherwise). The ring's hit holds the
  body's knock-down cooldown for the call, so a knock-down already pending from another hit
  that frame is kept and the log doesn't claim one.
* **The dome** (`src/ale_fx.rs`, Sonic only, by node name):
  * sonic_grenade.emt has no rate and no initial count; its emit count (e7221f95, keyed
    27.6 -> 38.3 over 0.007-0.297 s) is read as the opening burst (28 particles, the rule
    the Frag's flash uses) and then as particles per second for its 0.35 s life (~11 more,
    `EMIT_COUNT_RATE`). With the burst alone the spread never got to open into the ring; with
    one particle at the start (and the rate) the first frames were a few flat sheets where the
    recording's dome is whole; per 30 fps frame (~300) it was a solid white blob.
  * Its perp quads face out along their direction (`PERP_OUT_CONES`, as the Frag's sphere
    emitter): a see-through dome with a brighter rim while its spread is 28-74 degrees, then,
    as it opens to 90 by 0.25 s, a ring of quads standing round the blast (from above, a ring
    with a dark middle). Lying flat in the emitter's frame (before), it was a flat streak.
    Tried and not kept: the quads lying along the cone's surface (a funnel flattening into a
    disc) - from above a filled disc, not a ring.
  * Its air field sonic_grenade_air.fld (a drag: a wind of 0.004 m/s, Approach 0.80 -> 0.15
    by 0.43 s) stays off. Applied 6 times a second (the first version's fit to the early
    half-widths) it held the particles in a ball: 18-26% of the dome's brightened pixels fully
    white (the recording 0-2%), and the ring only 5.3 m out at +0.6 s where the recording's is
    past 6.3-9.5 m. Off: 8% white on the first frame, 0-2% after (sdm_e34, `s81/sat.py`), and
    it spreads faster. The early dome is still wider than recorded (1.4-2x at +0.07-0.2 s, the
    tester's measure with the field; wider without it): the quads' own size, 0 -> 5.5 m over a
    quarter of their life, sets that width.
  * sonic_grenade_flash.app's godrays (godray.tga, aspect 0.5 -> 15.8) stand vertical from
    their first frame, turned about the vertical to the camera (`VERTICAL_STREAKS`); along
    their motion (54-83 degrees out) they lay nearly flat as a white glare over the ground.
    The Energy's stun_hit_s godrays are unchanged.
* `BF_GRENADE_LOG` prints, per body in reach, when the ring will reach it and when it hits;
  `BF_COMBAT_LOG` the damage.

Verified (15 fps; scratchpad `s81/`):
* Test map, `--test BF_TEST_GRENADE_TYPE=Sonic BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.4
  BF_TEST_INSTANT_KILL=0 BF_TEST_TARGET=1 BF_CAMERA_PITCH=-0.3 BF_CAMERA_DISTANCE=6
  BF_VIEW_YAW=0.9 BF_GRENADE_LOG=1 BF_COMBAT_LOG=1` (`W1`): blast at t 2.80; due times
  Brutus +0.28, Tex +0.43, Hawk +0.46, Flint +0.51 s, hit on the next 15 fps frames (+0.33,
  +0.47, +0.47, +0.53 s; the first version hit a frame early); Brutus 40.6 HP at 4.3 m, Tex
  (the thrower) 0.1 at 5.0 m, Hawk 18.5, Flint 3.4; nobody floored, no "knocked down" lines.
  With `BF_TEST_THROW=1.5,0.15` (`W2`): Tex 7.8 HP at 4.2 m. After `TINT_MIN` (`W4`): the
  same hits, and Tex's 0.1 HP comes with no tint.
* sdm_e34 (`BF_MAP=sdm_e34 BF_TEST_GOTO=-44.4,15.5,-44.4,16.3`, the same throw and camera),
  side by side with the recording's frames 449-509 at matching times (`side_fina/b.png`): a
  see-through dome with a brighter rim from the first frame (the recording's shape; bigger),
  then the ring and the vertical shafts to ~+0.8 s, the decal from +0.8 s. Tex (the thrower,
  4.9 m) takes 1.1 HP and the tint (the recording: nothing at ~5 m; a thrower 0.1 m farther
  gets no tint, `TINT_MIN`). The 0.4-0.67 s pale
  white-blue on Tex the tester saw (the field version) is gone; the light effect is main's.
  **Doesn't match**: the demo is whiter and brighter overall than the recording's dark blue
  (the shared additive rule, #104); the shafts are several broad pale columns where the
  recording has two narrow ones, one upright and one leaning ~15 degrees out (the data's
  ~45 a second, 1.1-3.5 m wide quads); the ring is a soft wall of quads, not a thin flat
  band; the decal is hard concentric rings, ~3.8 m, seen from +0.8 s, where the recording's is
  a soft oval of ~2.2-3.3 m (an earlier measure said ~2.5x Tex's height) first seen at ~+1 s -
  left at the Frag's half-size reading and delay.
* **The look and the additive blend (#104).** The dome too wide early, its saturation, the
  characters inside it turning pale grey-blue and the broad godray columns all come mostly
  from how added sprites are summed. The demo adds each sprite's colour as linear light: over
  a dark ground a sprite's faint soft edge (a stored 0.02) shows as a stored ~0.15, so the
  5.5 m quads' and the godray texture's wide faint margins show where the console's would
  vanish. A test (not kept) turned each Sonic sprite's colour x alpha x texel into linear
  light first, as a stored value (exact for one sprite over black): on sdm_e34 the added
  colour came to 0.53-0.62 : 0.75-0.79 : 1 (the recording's 0.53 : 0.77 : 1; linear: 0.70-0.77
  : 0.85-0.91), no white pixels, the godrays narrow and Tex unlit - but far too faint, because
  overlapping sprites then sum in linear light (N sprites give N^0.45 x one, where the console
  gives N x one): a thin pale dome, the ring and shafts nearly gone (scratchpad
  `s81/side_dka/b.png`: recording, linear, stored per sprite, over the darker floor at
  -78.4,0.6). Matching the console needs the added sprites summed in stored values (e.g. drawn
  to their own buffer and added as stored values), a renderer change for every added sprite:
  left to #104. Until then the Sonic stays on the shared linear rule.
* Frag, Energy and Gas, pixel-diffed against main's build (the Frag command in **Verified**
  above, with each type): Frag 9.3k pixels over 75 frames (main against itself 7.7k), Gas 5.5k
  (5.6k), Energy 18.6k (8.7k) - small scattered differences in the Energy's random bolts; the
  frames look the same.

## Energy grenade (`src/bin/play_energy.rs`)

"Releases a maelstrom of charged electrical bolts upon detonation." Its blast is the others'
(fuse 1.75 s from landing; effect type h_e1f6c1e6: stun_grenade_master + stun_hit_s, sound
f875b6c6, no light; decal h_f4d65518), but its damage comes with its bolts, not 0.1 s after.
Damage-type 6 is DTYPE_PARTICLE (the XBE's damage-type table at 0x3bd9d0: 0 NORMAL, 1
BALLISTIC, 2 BLADED, 3 BIOREACTIVE, 4 GAS, 5 FLAME, 6 PARTICLE, 7 SONIC, 8 LASER, 9 PSYCHIC, 10
EXPLOSION, 11 POWERBLADE; the Gas's 4, the Sonic's 7 and the Frag's 10 fit).

* **The bolts, as the data has them**: stun_grenade_master's emitter throws ~6-7 particles in
  its first 0.33 s at 3.84 m/s, each carrying a "stun_grenade" effect: a ribbon (ARCb,
  blue-white, ~0.59 m wide) through ~26 particles laid over 0.3 s by an emitter whose offset
  sweeps 5.8 m out along its z in 0.2 s, jittering sideways and climbing, each drifting 1.6 m/s
  for 0.49 s. So a bolt is a jagged strand reaching ~6 m out from a head crawling out at
  3.84 m/s, gone ~0.8 s after it starts (3.84 m/s x 0.8 s + 5.8 m is about the 9 m radius).
  The recording: 3-8 strands crawling out from +0.1-0.18 s to +0.77-0.83 s.
* **Bolts to bodies** (an inference: whether the game's bolts seek bodies or are random bolts
  joined to bodies near them, the recording doesn't settle): every body within the 9 m radius,
  living or dead, the thrower too, draws a bolt of its own from the blast toward it, from
  +0.13 s (Hawk, at the grenade, died at +0.13 s). Its head crawls out at 3.84 m/s and stops
  where the bolt's reach (5.8 m, scaled down to the distance for a nearer body) ends on the
  body; it strikes when its reach gets to the body's side (0.4 m short of its middle), within
  the 0.2 s sweep for a body within ~6 m, later out to ~1 s at 9 m.
* **A strike**: the thrower takes 0.34 x Damage max (33 HP: Brutus at 0 m lost 31.5% of his
  105 HP in one step at +0.2 s, the one measurement; the Frag's 0.2 x the roll would be
  7.8-19.5 here) at any distance within the 9 m: it doesn't fall off, so a thrower 8.5 m out
  still takes 33 while a squadmate beside them takes ~5 (only the 0 m case was measured). The
  thrower stays standing, as Brutus did. Either is multiplied by the character's difficulty step
  (the thrower's measured share with STANDARD's x0.6 divided out, as the Frag's: **Damage**) and
  factor for damage-type 6 (**Damage types** above: Flint x2, the others 1); anyone else takes Damage max falling to
  nothing at the radius (the other grenades' rule; instant kill on the test map) and, if alive
  and not in the air, is thrown down back and up (3 m/s back, 4 m/s up: the demo's amounts;
  the recording's squadmate went up and back at +0.6 s, was down ~1 s and got up). A corpse
  (or a body already down) is thrown up and out: every body of its ragdoll brought up to 3 m/s
  along the throw (`Ragdoll::fling`, #259, [ragdoll.md](ragdoll.md); the recording threw a corpse
  into the air; 3 m/s lifts it only ~0.1-0.4 m). The player's red tint comes with their strike.
* `BF_GRENADE_LOG` prints each bolt (to whom, how far, alive or dead), each strike (how long
  after the blast) and whether a corpse's shove hit; `BF_COMBAT_LOG` the damage and whether
  they were thrown down.

Verified (test map, 15 fps; scratchpad `en74/`):
`--test BF_TEST_GRENADE_TYPE=Energy BF_TEST_GOTO=0,10,0,10 BF_TEST_TARGET=1
BF_TEST_THROW=1.5,0.6,5.5,0.6 BF_TEST_TOGGLE_KILL=5.0 BF_CAMERA_PITCH=-0.45
BF_CAMERA_DISTANCE=8 BF_GRENADE_LOG=1 BF_COMBAT_LOG=1`: the first blast kills Brutus (instant
kill) and floors Hawk; instant kill off, the second (t 8.67) strikes Flint (2.3 m, 73 HP, thrown
up and back, down ~1 s, back up) and Hawk (2.7 m, killed) at +0.33 s and Brutus's corpse (6.9 m,
"body thrown") at +0.33 s. Side by side with the recording's fourth blast (frame 1772, every
4th frame to +0.87 s): the cyan wash at +0.07-0.2 s, the bolts from ~+0.27 s, strongest
+0.33-0.6 s, gone by ~0.8 s as recorded; strands end on the struck bodies. Differences: the
godray shafts show from ~+0.07 s to ~+0.33 s, the recording's from +0.27 s to +0.47 s; the
wash is whiter and the strands thinner and crisper than the recording's (its halo is wider);
some of the master effect's random bolts climb a few metres into the air (its emitter tumbles
them; the recording has a few such).

## Thermo (#82)

"Powerful timed explosive charge. Detonates 15 seconds after armed." Defined in m09_b alone
(h_0141ea85, its label h_e9c9f9d2 "Thermo"): `timer` 15, group-type 3 (IG_EXPLOSIVE) item 2 (it
comes between the Sentry (3, 1) and the Roller (3, 4) in the store's order), h_1ee2f4ed 2
(IOU_PLACE_ON_GROUND), function-type 0 (no item set of its own), stack-limit 5, mass 23, its HUD
icon h_0ebe1a32; its explosion h_08f5cb41: Damage 19.5-97.5, damage-type 10 (EXPLOSION),
radius 3, core true. The level places it once (h_e04e05e1 at (-178.4, 58.3, 126.5),
levels-m09_b.xml 938), and not as a pickup: it's set down live (h_1cb7d5f2) on the enemies' side
(item-alignment 1), as the tutorial's mines and e16's, e34's and e40's Sentries are. The item
object's handler (0x14a0a0) lets it go as a throw does (+0x1bc bit 1, its item set's +0x10,
FUN_0014a150's steps). Nothing in the level refers to it (no trigger, script or objective): a
prop that goes off on its own. The leads and the takes are `todo/82-thermo-grenade/`.

* Nothing of its own to build: it goes the placed types' way (**Placing** above). G sets it down
  at the feet with the stance's place_hi clip (the count drops at the press), its fuse runs
  from when it's down (the grenade's contact handler 0x149510 starts it, as every type's), and
  15 s later it goes off with its effect type's ALE effects, light and sounds, its decal, and
  the shared blast's damage (#112). Function-type 0: the base item set (0x39bec0), whose slots
  are stubs; no detonator (IFSET_DETONATOR, 17, is on no item).
* **The level's charge** (`place_mines`, play_sentry.rs): set down where it's placed as the level
  loads, team 1, owned by nobody, its fuse armed (its body settling on the floor is a first
  contact: in take01 it lies 2-3 cm off its placement), counting while no cutscene holds the world
  (**The fuse**). m09_b's startup cinematic (m09_b_startup.gscr, 12.035 s) sets the cutscene bit
  with its 122 at 0 s and clears it with its 123 at 9.0 s, so the charge goes off **24.0 s after
  the level's start**, wherever the squad is (take01: held at 15.000 for 6.3 s of its snapshot's
  clock, which was taken ~2.7 s into the cinematic, then 15.0 s to the blast at 21.3 s). With
  `BF_TEST_GOTO` or `BF_SCRIPT` (no start scripts) it goes off 15.0 s after the load. Going off it
  sends DESTROYED as the level's mines do (nothing listens). The rule is every live grenade item
  the level sets down, not the Thermo's: the Sentries as before, but not a Roller (m09_a's one is
  left out pending #316, see **Roller**); a spawn-trigger's template stays out of the world (none
  of the Sentries is one).
* **Its blast**: the effect type h_fbdd1258 has the Roller's h_013b3ca8's content (the Frag's six
  ALE effects, light_explosion, h_065168c9), at 3 m, plus the impact sound h_16611581; no
  per-material ground puff (its explosion's h_0cbcb8f2 is empty, where the Roller's has dirt and
  stone entries; the demo draws none for any grenade yet).
* Blocked (not filmed, not traced): whether a shot sets it off (take03's round landed ~0.3 m from
  it and it didn't go; its 1 hp and the item's damage path aren't read), so it isn't among the
  shots' targets as the Sentries are; its damage to enemies by distance and to squadmates (only
  the thrower at 1.4 m is measured, take04: Brutus -13.9, the core blast's 0.42 of a roll by the
  code, #112's); its look (the mesh h_0aac8518 is never seen lit in the takes) and sounds
  (h_1d85a923's reader isn't traced); the AI's reaction to it.
* Its kit is made where the level defines it: `load_kits` takes in the grenade and explosive types
  a level puts down itself (its item spawns, h_1d1d4f05, as pickups or set down live), so m09_b
  loads the Thermo (`8 grenade types: Frag, Gas, Sonic, Energy, Light, Sentry, Thermo, Roller`).
  The rule is general, not the Thermo's. Run on each level that puts down a type outside the
  squad's seven (28: e16, e23, e34, e40, m05_c, m07_c, m08_a, m08_b, m09_a-c, m09_x, m14_b, m14_c
  and the fourteen arenas; `BF_MAP=<level>`, the kit list it prints), it adds a kit on four:
  m09_b's Thermo, and the Sentry h_f73de83d on e16, e34 and e40 (6, 6 and 5 types, one more each
  than before). On the others what they put down is loaded already under its label (another
  definition of the Sentry or the Roller) or isn't a usable item. Those three levels lay that
  Sentry as live mines, set down with h_1cb7d5f2: e16 14, e34 6 and e40 7, item-alignment 1 (the
  enemies') but one of e34's, -1. With its kit made, `place_mines` (play_sentry.rs, #228) now arms
  them as the level loads, as the game's handler does (0x14a0a0): `BF_SENTRY_LOG=1` on e34 prints
  six "the level's mine ... set down armed" at t 0.00, h_0a8cb601 on team 0 and five on team 1, the
  enemies', which go off for Brutus, Flint or Hawk within their 3 m and for Tex only stepping on
  one, as the tutorial's do. Before #82 they had no kit there and weren't placed. Not checked
  against footage of those levels.
* **A live mine of a type that shares the squad's label has its own kit (#317).** `load_kits`
  took a level's own types by label, so a Sentry defined under another hash than the squad's
  h_e5f1f063 got no kit wherever the squad's own was loaded, and the level's live mines of it
  were never put down. It now also loads the type of each live-placed item (`Placement::placed`,
  h_1cb7d5f2) by its own hash (not a spawn-trigger's template; a Roller stays #316's), after
  the label rule, so the squad's kit keeps its place first and `GrenadeKits::find("Sentry")`
  still finds it. m09_b's live Sentry h_f73de83d is set down (`9 grenade types: Frag, Gas,
  Sonic, Energy, Light, Sentry, Thermo, Sentry, Roller`, the second Sentry sorted by its
  (category, index) after the Thermo), m09_x's four live Sentries (`8 grenade types`: one of the
  squad's type, which was set down before, and three of h_f73de83d) and e23's three, of h_ecfe0d8c (`6 grenade types: Frag, Gas, Energy,
  Sentry, Roller, Sentry`; function-type 8, explosion h_01f142eb radius 4 as the squad's). Each
  mine is its own kit's: `place_mines` takes the kit by the object's type hash, so its radius
  and blast are its definition's. `BF_SENTRY_LOG=1` on every map before and after (53 runs,
  12 frames each): the placed lists are the same on all the others (e05 4, e10 6, e16 14,
  e34 6, e40 7, m03_a 5, m03_b 5, m03_c 4, m08_b 14, m14_a 7, m14_b 1, m14_c 2, the tutorial 3,
  ...), only e23 (0 to 3), m09_b (1 to 2) and m09_x (1 to 4) change. The squad's start
  stacks are credited by label (`StartStore::grenades`), so with two kits of one label the
  first, the squad's, takes them and the others start with none (else the stack would be
  carried twice): `BF_INVENTORY_LOG=1` on e23, m09_b, m09_x and e34 shows no Sentry stack
  (none of those zones carries one), and m14_c's `Sentry 10` is as before. Not checked: the new
  mines going off. Tex can't walk to them from the start (held at z -93.5 short of e23's
  first, (-67.3, 25.4, -97.9), and at z 78.3 short of m09_b's, (-125.5, 57.5, 86.9)), and
  `BF_TEST_HOSTILE=hawk,4,0,1` didn't set e23's first off (no check line at all, not looked
  into).
* Blocked: how the squad comes by one. The data has no Thermo pickup (its one placement is
  the set-down charge above): pickups are #17's, disarming a set-down item #145. Also blocked:
  what m09_b's mission does with the charge (its objectives and triggers, the campaign). Until
  then the squad has none but by the test hooks.

Checked (`BF_GRENADE_LOG=1 BF_COMBAT_LOG=1`): on m09_b (`-- --test BF_MAP=m09_b BF_CHARACTER=3
BF_TEST_INV=Thermo:3 BF_TEST_GRENADE_TYPE=Thermo BF_TEST_THROW=2.0,0.1`; m09_b starts in a
letterbox that hides the HUD: `BF_TRIGGER_FIRE=trigger_letterbox468,USER_SIGNAL1,0.05` lifts it,
the tester's) the press at 2.0 s drops the count 3 -> 2, the clip lets it go at 2.53 s, it's down
0.6 m from Tex at 2.93 s and goes off at 18.00 s (15.07 s, a 15 fps step), Tex 1.1 m from it taking
his share (14.7). On the test map with m09_b's data
(`BF_TESTMAP_LEVELS=mp_common,mp1,mp2,mp3,mp4,mp6,mp7,mp8,m02_a,m09_a,sdm_e34,m09_b`, the same
hooks, `BF_TEST_GOTO=0,30,0,60 BF_TEST_GO_AT=100 BF_TEST_INSTANT_KILL=0`, dm Shrike Soldiers frozen
1.8, 3.5 and 3.9 m from where it lies): the one at 1.8 m takes 97.5 x (1 - 1.83 / 3) x 1.4 = 53.2
(its stamina shield 25 of it), the two beyond 3 m nothing. The item box shows its icon, "Thermo"
and the count.

Checked (#82, `BF_GRENADE_LOG=1 BF_SENTRY_LOG=1 BF_SCRIPT_LOG=1`): `BF_MAP=m09_b` from the level's
start (no `BF_TEST_GOTO`): "the level's grenade h_e04e05e1 (Thermo) set down armed at [-178.37,
58.25, 126.47], team 1" at t 0.00 (take01's object at (-178.386, 58.273, 126.433)); m09_b_startup's 122 at
t 0.00, its 123 at 8.97 (script 9.000), and the blast at **23.97** (9.0 + 15.0), with 6 effects,
light 0f220d41, h_065168c9 + 16611581 and the decal f5ccedb0. From take02's spot
(`BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_GOTO=-184.22,117.80,-108.7,183.4 BF_TEST_GO_AT=100
BF_CAPTURE_FPS=15`, Brutus 10 m from it facing it): the blast at 15.07 (15 s after the load, no
start scripts), Brutus untouched; frames 220-249 beside take02's 10.7-12.6 s: the same doorway, a
white core at the floor line on the first frame (the take's 11.0-11.15), the yellow-white
fireball filling the doorway at +0.13-0.4 s (11.35-11.5), orange fire and brown smoke at +0.6 s
(11.7), an orange haze after; the demo's haze is dimmer and gone sooner than the take's (>2 s),
the shared effects' rendering.

Test hooks (with `BF_TEST_GOTO`, on the test map or a map):
* `BF_TEST_GRENADE_TYPE=<label>` (any case, e.g. `Sonic`) selects that type at the start,
  giving a stack of it if none is carried.
* `BF_TEST_THROW=<s>[,<hold s>][,<s>,<hold s>...]` holds G from s for hold s (default
  `CHARGE_TIME`, 0.625 s: a full charge), as many times as given (a placed type: one per press). At
  15 fps a hold under one step (0.067 s) can fall between steps.
* `BF_TEST_NEXT_GRENADE=<s>[,<s>...]` taps Tab (0.1 s) at each time: the next item (#196; it
  pressed the T key before, which stepped the grenade types only).
* `BF_TEST_DETONATE=<s>` sets off every grenade out at that time (through the fuse: a cutscene
  holds it). (The Sentry's own hooks: see **Sentry**; the Roller's `BF_ROLLER_LOG` and
  `BF_ROLLER_RAYS`: see **Roller**.)
* `BF_TEST_EXPLOSION=1` sets off the selected type 6 m ahead every 1.5 s.
* `BF_TEST_DROP=<s>[,<s>...]` sets off the selected type at the controlled character's feet at
  each time (to stand in a Gas cloud). `BF_GRENADE_LOG` prints each cloud, when it's over and,
  per character it reached, when it poisoned them, how much and at what rate.

Verified (captures at 15 fps in the scratchpad; the reference frames are the user's xemu
recordings at 60 fps):
* Frag, test map (`--test BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.3 BF_CAMERA_PITCH=-0.5
  BF_CAMERA_DISTANCE=9 BF_VIEW_YAW=0.2`): button-up 1.8 s, leaves the hand 2.40 s, down 0.13 s
  later 3.2 m ahead, blast 4.07 s: 2.27 s after the button (recorded 2.28-2.38, input to
  picture: the game's ~2.1-2.2 s once the lag and input delay are taken off, #140, so the demo's
  is likely ~0.1 s late, by the release above; the fuse itself fits), 1.67 s after leaving the
  hand (recorded 1.65-1.78, picture to picture). Side by side with frag/a from its flash (frame 693) at
  matching times, 0-1 s: a white haze on the blast frame; the red tint two capture frames later
  (the recording's 701), as Tex 6.6 m away takes 12.6 of 115; the fireball turns orange at
  ~0.4-0.5 s and brown after, with no second white flash; a dark 6 m scorch after. The demo's
  blast is farther from the camera, so smaller on screen; the recording's smoke spreads wider
  and darker. Four captures (three run at once) give the same bright-pixel counts frame for
  frame.
* Sonic: see **Sonic grenade**.
* Light (`BF_TEST_GRENADE_TYPE=light`): see **Light grenade** above.
* Hand-over during a throw (`--test BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.07
  BF_TEST_SELECT=1,1.6 BF_GRENADE_LOG=1`): the count drops to 9 at the button, control passes at
  2.07 s, before the grenade would leave the hand (2.20 s without the hand-over), and
  `BF_GRENADE_LOG` prints "t 2.07: throw cut short, grenade back (10 now)": nothing is thrown.
  The recipe asked for the select at 1.1 s, which no longer shows it: the hand-over comes 0.5 s
  after the ask (`SELECT_TIME`, 0.6 when this was written), at 1.6 s, as G is let go, and
  depending on the frame it lands in the throw goes ahead (the tester's runs, on main's build and
  #161's) or never starts (a run at 15 fps here).
* Sentry (`BF_TEST_GRENADE_TYPE=Sentry`, ten presses 1 s apart): the box counts 10, 9 ... with
  no digit at 1, then shows Frag 10; `BF_TEST_NEXT_GRENADE=12.5,13` steps to Energy, then Gas.
* Roller: see **Roller** for its seeking and roll. Its sound (take09's staging, `BF_SOUND_LOG=1`,
  30 fps): down at 2.00 s, h_19dbc65d at 2.00, 3.00, 4.00, 5.00 s (0.97 s on the 30 fps step),
  0.80, 0.80, 0.53, 0.20 as it goes away.
* Gas, test map, standing in the cloud (`--test BF_TEST_GOTO=0,4,0,3.5 BF_CAMERA_PITCH=-0.15
  BF_TEST_GRENADE_TYPE=Gas BF_TEST_DROP=2 BF_GRENADE_LOG=1`): blast at 2.07 s, Tex poisoned from
  +0.13 to +5.60 s (the first 15 fps step past +0.083 s), 97.5 HP at 17.8 HP/s, health 115 ->
  17.5. As Flint (`BF_TEST_SELECT=1,0.5 BF_TEST_DROP=3`): 0.9 HP/s
  (x0.05). Side by side with the recording's second cloud (gas f844 on, by time since the
  blast): the cloud thickens by 1-1.5 s and from 1.5 s veils Tex and the ground olive as
  recorded; the recording's is darker and browner (over a darker map) and its big white
  flash flickers for ~1 s where the demo's is one short burst. The demo's cloud still hangs at
  6.5-7 s (the data's puffs live 2.3-4.4 s from an emitter running 4.3 s); the recording's
  first cloud is half gone by ~6 s (low confidence: Tex walks out of it then). From 12 m the
  cloud is ~6.5 m wide and ~5 m tall (recorded ~6 x 4 m, low confidence).
* Death in the cloud with a squadmate in it (`--test BF_TEST_TARGET=1 BF_TEST_INSTANT_KILL=0
  BF_TEST_GOTO=0,4,0,-3 BF_TEST_HEALTH=40 BF_TEST_GRENADE_TYPE=Gas BF_TEST_DROP=4
  BF_GRENADE_LOG=1`): Tex dies at +2.40 s (41.1 HP at 18.1 HP/s); Brutus, standing in it, is
  poisoned from +0.13 to +5.60 s, 97.5 HP at 17.8 HP/s, through the death camera and the
  hand-over (the whole 5.5 s at the rate: no gap, no catch-up).
* Frag and Sonic with the field allow-list (the Frag command above, and with
  `BF_TEST_GRENADE_TYPE=Sonic`), pixel-diffed against main's build frame for frame: the only
  differences are the trail's puffs (around the hand and along the flight, and the Frag's
  rising trail column lit by its blast); the Sonic's dome and ring and the Frag's blast are
  identical (before #81, which changed the Sonic's).
* Energy: see **Energy grenade**.
* No crosshair of the grenade's own (#119; `BF_MAP=sdm_e34 BF_TEST_GOTO=-44.4,15.5,-44.4,15.5
  BF_TEST_THROW=1.5,2.0`, 75 frames): the gun's crosshair keeps the same box (HUD-blue pixels
  x 306.7-334.7, y 177.7-207.0, centred on (320.7, 192.3)) on every frame, before, through the
  2 s charge, the release and the meter's hold; no blue pixel outside its first-frame mask in
  the window x 280-352, y 155-230;
  the meter's outline x 354-368, y 162-222 from 1.53 to 4.07 s. Side by side with `todo/119`
  take01 f138, f170, f264, f286, f313 (the Therm Sweeper's arcs stay, the meter beside them in
  the same place). Edge cases: a hand-over mid-charge (`BF_TEST_SELECT=1,2.0`) shows the new
  member's crosshair (Flint's) and nothing else; a death mid-charge (`BF_TEST_DIE=2.5`) hides
  the crosshair from the death frame; a weapon switch (`BF_TEST_SWITCH=1.5`) still hides it
  from the drop to the grab; a Roller press shows no meter and keeps the crosshair. In the scope
  (`BF_TEST_WEAPON=L-Shot-75 BF_TEST_AIM=1`: the autopilot aims from `BF_TEST_AIM`, and a
  zooming gun scopes; `BF_TEST_SCOPE` isn't read under the autopilot) the crosshair stays at
  (320, 239.7) through the charge and the throw, and since #162 the meter beside it (the fill
  x 355.3-366.0, y 212.0-266.7 at frame 40: the unscoped place + 48, the corner at (352, 208));
  the game's scoped throw isn't filmed, and the demo's throw doesn't leave the scope as the
  game's LT release does (a follow-up).
* The meter's fade and inside (#162; `BF_CAPTURE_FPS=30 BF_MAP=sdm_e34
  BF_TEST_GOTO=-44.4,15.5,-44.6,15.1 BF_TEST_THROW=1.5,2.0 BF_CAMERA_DISTANCE=12`, the meter
  over the dark wall panel): the count drops at frame 106, the meter is full through 121 and its
  fill's strength over the wall from 122 on is 0.80, 0.64, 0.50, 0.39, 0.28, 0.21, 0.15, 0.10,
  0.05, 0.04, 0.02, then gone (the take, #119 spec §5.2: 0.80, 0.66, 0.51, 0.42, 0.33, 0.24,
  0.15, 0.11, 0.07, 0.04, 0.02; the code's curve 0.81, 0.65, 0.51, 0.39, 0.30, 0.22, 0.15,
  0.10, 0.06, 0.04, 0.02). The first faded frame is 0.533 s after the count drop (the take's
  0.517). The ground (wall, Tex's arm) shows through the box above the fill while it charges,
  as take01 f170's sand does. The outline keeps its colour (brighter and harder than the
  take's, out of #162's scope). The Jax-iR
  (`BF_TEST_WEAPON=Jax-iR`) keeps its own crosshair (h_0ef11ee8) through a charge.
