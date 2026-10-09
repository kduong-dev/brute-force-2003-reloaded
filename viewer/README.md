# Brute Force character viewer (Rust / Bevy)

Four programs:

| Binary | What | Run |
|---|---|---|
| `bf_viewer` (default) | character / animation viewer, reads the **original game files** (`../Brute Force/data/*.tgz`) | `cargo run` |
| `bf_play` | **playable demo**: run around as Brutus, Flint, Hawk or Tex with the game's own locomotion | `cargo run --bin bf_play` (the test map: `cargo run --bin bf_play -- --test`) |
| `bf_level` | level viewer: terrain, placed objects, sky and fog of a level (default Battle of Bulgar) | `cargo run --bin bf_level -- [level]` |
| `glb_viewer` | viewer for the exported `../decompiled/characters/*.glb` | `cargo run --bin glb_viewer` |

## Playable demo (`bf_play`)

| Keyboard / mouse (no gamepad for now) | Action |
|---|---|
| WASD | move (relative to the camera); the character turns toward the direction |
| Shift | sprint |
| Ctrl | walk |
| Right mouse (hold) | aim: the camera dollies in (as when firing), the gun is turned onto the crosshair while the legs keep moving; moving away from the aim backpedals. Moving across it (30-150 degrees off the crosshair) side-steps, standing (`Sc_w1/w2_rp_f_side_walk_l/r` up to 100 degrees, `rp_b_side_walk_l/r` past it) or crouched (`cr_f/b_side_walk_l/r`). Those clips twist the upper body about a quarter turn off the root, and their root motion runs straight along it, forward or back, as far as the walks. So the character turns until the gun is on the crosshair and walks where the clip takes it: across the aim, a little forward or back. `BF_TEST_STRAFE=<x>[,<y>]` moves that way (camera-relative) with `BF_TEST_GOTO` |
| Left mouse (hold) | fire the held weapon (at its own rate; also aims while held). Standing, the feet turn with the aim (the spine keeps at most ~11° of twist, also when looking down and turning). As in the game, the camera snaps in on the first shot, holds ~0.7 s after the last, then eases back out; looking steeply down it stays out |
| Q | switch between the character's two weapons (animated: stow one, draw the other; plays the game's hard-coded switch sounds ff820fda / e1e97460, 0.65 s apart) |
| Space | jump: standing jumps crouch first, running jumps launch straight away and carry the run's momentum (Brutus uses his four-legged jump when sprinting on all fours) |
| C | dodge roll / sidestep to the left or right of the movement |
| Z | crouch / stand. Standing still: kneel (the stance's `stand2crouch`, then `rp_crouch_idle`; `crouch2stand` to get up). Moving: the crouch walk (`Sc_w1/w2_cr_walk`, `cr_back_walk` backing off while aiming; the clips' own root motion and footstep events), straight in from a run. Jump, dodge or sprint stands up. `BF_TEST_CROUCH=<s>` crouches from that time with `BF_TEST_GOTO` |
| M | cycle the ground surface (changes footstep / landing / slide sounds) |
| click, then mouse | look around (Esc releases the mouse) |
| wheel | zoom |
| 1-4 | take control of Brutus / Flint / Hawk / Tex: their name pops up in the middle of the screen in the game's own HUD font (atlas e0afcd52), with three red chevrons (the game's chevron texture 158f87c1) lighting up on the radar toward their portrait and the HUD open sound; at the cut the close sound plays and the name zooms out to 3x and fades (0.25 s, as in the capture), and the character you left drops back into the squad AI with a follow line ("I'm right behind you", "Following at a distance"...: chatter set f415ab02, from the shared voice bank `ml-sounds/en/common-en.tgz`; their speech icon shows by their portrait while they talk). Not while dead: the death camera hands over by itself |
| R | reload: the stance's reload clip (`Sc_w1_reload` / `Sc_w2_reload`; upper body on the move, whole body standing); automatic on an empty clip. The clip reads 0 (panel red) until the clip's magazine-in event |
| G | use the item in the item box. A thrown grenade (Frag, Energy, Gas, Light, Sonic): hold to charge (the orange meter right of the bracket reticle, full in 0.6 s), let go to throw - the charge sets how far; the count drops at once. A placed one (Roller, Sentry): set down at the feet at the press. See **Grenades** below. The main game starts with 3 Frags |
| E (hold) | use: a gate's wall panel, from in front of it within 2.5 m, looking at it: "Hold E to activate panel." shows and a blue ring marks its button; held 0.5 s, the gate opens and stays open. See "Gates and their wall panels" |
| T | the next grenade type carried (the demo's key: the recordings don't show the game's button for it) |
| Tab | the item box (the game's B button): tap for the next item carried (each grenade type, Medkit); hold for the item list, the wheel picks one. With a Medkit selected, G heals 80 ("No need to heal" at full health). See "Health pickups" |

### Test map (`cargo run --bin bf_play -- --test`)

A flat test floor for trying things out, only reachable with the `--test` flag. It opens
straight into play: no loading screen, intro or menu (`src/bin/play_testmap.rs`).

* **Every hand weapon on a rack**, floating and turning in front of the start: the 27 weapon
  definitions with a clip whose model loads (one per model), from the first mission's data,
  the multiplayer archives (`mp_common`, `mp1`-`mp8`) and m02_a, which holds the three
  campaign-only ones (A10 Bioreactive, Confed LZR-50, Jax-iP); about 0.7 s. Walk into one
  to take it into the held weapon's slot: the character is respawned carrying it, with a full
  clip, and "Took <name>" shows.
  * Loading every level (about 6 s) adds no others: the rest are the creatures' built-in
    weapons (no model), Feral Cutter (the same model as another) and the iKhan-GPL (a model
    under 0.1 m).
  * `BF_TESTMAP_LEVELS=<level>,<level>...` loads a different list.
  * `BF_TESTMAP_LOG=1` lists every weapon on the rack with its type, clip and model size, and
    every hand weapon left off it, with why.
* **One of every pickup type** with a model (one per model) in a grid behind the rack.
  * They're dropped in from 0.3 m, upright as modelled but turned at random and tipped up to
    0.2 rad, so each settles as it would. A crate lands on its base; the Garo fruit (modelled
    on its point) and the cards (on their edges) fall over onto their sides.
  * They're inventory-objects as on a map, so medkits and fruit work as in "Health pickups",
    and everything is loose (kicked, thrown by blasts, tumbling).
* **Every squad grenade type**, a full stack of each (its stack-limit, 10): Frag, Energy,
  Gas, Light, Sonic, Roller, Sentry. m09_a is
  loaded for the Light (only in m09_a/b/c/x). T steps through them. See **Grenades**.
* **Instant kill**, on at the start (`BF_TEST_INSTANT_KILL=0`: off), K toggles it (`BF_TEST_TOGGLE_KILL=<s>[,<s>...]` presses it at those times). The player's shots and grenades kill any
  squad member they hurt, in one hit. The player still takes normal damage.
* **X kills the controlled character** on the spot, as any hurt does: the death cry and
  ragdoll, then the death camera and the hand-over to the next squad member (or, with nobody
  left, the camera stays on the body). `BF_TEST_SUICIDE=<s>` presses it at that time.
* **The controls panel**, which the main game no longer shows: the keys and the debug line
  (character, state, clip, surface, weapon). H hides it.

### Grenades (`src/bin/play_grenade.rs`)

Every grenade type comes from the loaded levels' `inventory-grenade` definitions (read into
`WeaponDef`, `src/bf/weapon.rs`); nothing per type is hard-coded except which definitions the
squad carries when several share a label (the Sentry has three, the Roller two: the ones the
recordings show, `SQUAD_GRENADES`) and the order T steps through them. The squad carries only
those seven. The Molotov (e01's h_f42e0faa, a labelled grenade with an icon) is the mutants'
weapon (#78), not the squad's: its definition is read like the others' for an AI thrower to
use. Per type:

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

`BF_GRENADE_LOG=1` prints each type's fields at load (`grenade <label> ...`), each grenade's
first contact (how long after leaving the hand, how far from the thrower), each blast and
decal, and a throw called off.

* **Throwing** (thrown types): hold G to charge. The meter (the recordings, every grenade): a
  blue outlined bar at x 354.0-367.7, y 162.0-221.7 beside the reticle brackets, its fill
  x 355.3-365.7 growing linearly from y 218.7 to 164.3 in 0.6 s, sRGB 255 160 53 at 0.65, a
  dark divider at y 179; let go, it holds its level 0.5 s and goes (the recordings' 0.1 s
  fade is drawn as a cut halfway through it). The count drops at the button (the recordings:
  one game frame after it). The stance's throw clip takes the grenade in hand at its reach
  event - its trail (grenade_trail's smoke, riding on it) and the trail effect type's sound
  (the Frag's hiss, ~0.33 s after the button in the recordings) start then - and lets go at
  its release event (0.60 s after the button for Tex, as recorded). It flies at 12-20 m/s over
  the charge, aimed 0.25 rad below the crosshair (at most 0.8 rad down), and keeps 0.3 of its
  speed along the ground per bounce: values fitted to the Frag recording (leaves the hand
  0.60-0.63 s after the button, goes off 1.65-1.78 s later, so down 0.13-0.28 s after leaving
  the hand, a few metres ahead with the camera looking down, and resting near there), not
  from the data. Bounces are
  silent; the trail keeps puffing where it lies until the blast.
* **Placing** (Roller, Sentry): the press drops the count and plays the event sound; the
  stance's `Sc_w1/w2_place_hi` clip takes it in hand at its reach event (0.23-0.47 s over the
  squad's clips: the recordings' "in hand 0.38-0.47 s") and lets go at its 19f8311b event
  (0.50-0.60 s); it drops from the hand at the feet (Tex's Roller is down 0.97 s after the
  press; recorded ~1.05 s). Squadmates don't dive from one set down. The Roller then rolls
  straight ahead at 4.7 m/s (4.4-5.0 measured), its object sound h_19dbc65d again every 0.97 s
  while it rolls (the recording's loop), full within 5 m and silent at 25 m (the sound's own
  `falloff 5` / `h_fd40e332 25`, sounds-mp1.xml); the fuse (25 s; the Sentry's 9999 s) runs
  from when it's down.
* **T** steps to the next type carried; it's ignored while a throw charges or is under way.
  A throw cut short (knocked down, or control handed over, before the grenade leaves the hand)
  puts the grenade back.
* **The blast**, on the ground below where it went off: the effect type's ALE effects and its
  light effect run once from that frame (`src/ale_fx.rs`; an effect without a parent now starts
  on the frame it's spawned; a "light_" effect is a point light: reach = its size, strength =
  colour x alpha x size, the DNA light's tuning), its sounds (h_f724cb8c read as a delay: the
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
    emitter's frame (the Sonic's ring and the laser hits' rings show that), which drew it as a
    stack of flat discs seen nearly edge-on: a wide flat streak. From a sphere emitter whose
    particles move out at 0.3 m/s or more, a perp quad now faces out along its own direction,
    and the fireball is round from its first frame, as in the recording. Grenade effects only;
    the 0.3 m/s threshold is the demo's.
* **Damage**: everyone within the radius takes Damage max, falling to nothing at the radius;
  the thrower takes 0.2 x a roll of Damage min..max anywhere inside it. That rule is the
  demo's, fitted to the Frag recording (six blasts, near and far: 12.5-13.3 HP of Tex's 115
  each, ~11%, no falloff); the game's formula isn't known. No damage from a blast without any
  (the Light); one whose damage is dealt over time (h_04ea9251 > 0: the Gas, whose recording
  shows none at once) leaves a poison cloud instead (see **The Gas's cloud** below). The
  Energy's damage comes with its bolts instead (see **Energy grenade**).
* **Damage types.** Each character's combat-target lists factors per damage-type
  (`<h_142be76f><h_1d403525 Type h_04653d86>`, read into `Game::character_damage_factors`;
  the shield's own list is empty): Flint takes type 4 (the Gas) x0.05 and type 6 (the Energy)
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
* **HUD**: the item box shows the selected type's icon (62.5 units square at (529.3, 376.4):
  the recording's icon spans x 553-568, y 384-425; at 50 units it came out 0.8x), label and
  count (count at the right middle, the digit at y ~410-421 as recorded; hidden while only one
  is carried, as in the Light and Sentry recordings). The medkit keeps its own icon size and
  place (50 units at (535, 379), fitted to the medkit captures). The meter's fill is drawn as (226, 148,
  54) at 0.65: what the console's stored-value blend of (255, 160, 53) at 0.65 gives over the
  recording's ground (63, 63, 55), since the UI blends in linear light. The charge sound is
  queued on the frame the meter first shows (`BF_SOUND_LOG` and the capture agree). When the selected type runs out, the first type still carried is
  selected (the Sentry recording: the last Sentry set down, the box shows Frag 10).

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
  times their damage-type factor (Flint 0.9 HP/s), from 0.083 s after the blast for 5.5 s
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
    more vertical than the recordings' - a partial match). Every other field - the Sonic's
    sonic_grenade_air.fld, the Frag's exp-lrg-air, every gravity field (exp-lrg-shrap,
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
| Light (#77) | phosphor_grenade + light_phosphor, ignition h_1080aaf1 and f5260e35 after 0.2 s, no damage, no decal, no tint, pickups left alone | checking the 30 s burn and the light's strength against the recording |
| Sonic (#81) | goes off on first contact (timer 0), grenade_sonic (dome, ring, godrays) + light_sonic_grenade, f36fb063, decal h_fb24bcb7 | the damage arriving with the ring (it's dealt at once) |
| Roller (#79) | set down (place_hi), rolls straight at 4.7 m/s with its rolling sound, bounces off walls, 25 s fuse, the Frag's effects + h_065168c9 | seeking |
| Sentry (#80) | set down (place_hi), lies there; exp-mine + light_explosion, h_145f09e5 | its trigger (9999 s timer: `BF_TEST_DETONATE` sets it off); arming, LEDs, disarming |

Not done for any: ALE fields other than the Gas cloud's and the trail's rise (exp-lrg-air,
the gravity and turbulence fields: #101); decals don't
follow uneven ground (a plane along the slope under the middle: a big scorch on a bumpy
hillside is partly buried); the sounds' play-length; distance falloff for blast sounds beyond
a volume.

#### Energy grenade (`src/bin/play_energy.rs`)

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
  thrower stays standing, as Brutus did; anyone else takes Damage max falling to
  nothing at the radius (the other grenades' rule; instant kill on the test map) and, if alive
  and not in the air, is thrown down back and up (3 m/s back, 4 m/s up: the demo's amounts;
  the recording's squadmate went up and back at +0.6 s, was down ~1 s and got up). A corpse
  (or a body already down) is thrown up and out with the ragdoll's shot shove, brought up to
  3 m/s along a line up through its pelvis (the recording threw a corpse into the air; the
  shove's 3 m/s lifts it only ~0.1-0.4 m). The player's red tint comes with their strike.
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

Test hooks (with `BF_TEST_GOTO`, on the test map or a map):
* `BF_TEST_GRENADE_TYPE=<label>` (any case, e.g. `Sonic`) selects that type at the start,
  giving a stack of it if none is carried.
* `BF_TEST_THROW=<s>[,<hold s>][,<s>,<hold s>...]` holds G from s for hold s (default 0.6: a
  full charge), as many times as given (a placed type: one per press). At 15 fps a hold under
  one step (0.067 s) can fall between steps.
* `BF_TEST_NEXT_GRENADE=<s>[,<s>...]` presses T once at each time.
* `BF_TEST_DETONATE=<s>` sets off every grenade out at that time.
* `BF_TEST_EXPLOSION=1` sets off the selected type 6 m ahead every 1.5 s.
* `BF_TEST_DROP=<s>[,<s>...]` sets off the selected type at the controlled character's feet at
  each time (to stand in a Gas cloud). `BF_GRENADE_LOG` prints each cloud, when it's over and,
  per character it reached, when it poisoned them, how much and at what rate.

Verified (captures at 15 fps in the scratchpad; the reference frames are the user's xemu
recordings at 60 fps):
* Frag, test map (`--test BF_TEST_GOTO=0,4,0,4 BF_TEST_THROW=1.5,0.3 BF_CAMERA_PITCH=-0.5
  BF_CAMERA_DISTANCE=9 BF_VIEW_YAW=0.2`): button-up 1.8 s, leaves the hand 2.40 s, down 0.13 s
  later 3.2 m ahead, blast 4.07 s: 2.27 s after the button (recorded 2.28-2.38), 1.67 s after
  leaving the hand (recorded 1.65-1.78). Side by side with frag/a from its flash (frame 693) at
  matching times, 0-1 s: a white haze on the blast frame; the red tint two capture frames later
  (the recording's 701), as Tex 6.6 m away takes 12.6 of 115; the fireball turns orange at
  ~0.4-0.5 s and brown after, with no second white flash; a dark 6 m scorch after. The demo's
  blast is farther from the camera, so smaller on screen; the recording's smoke spreads wider
  and darker. Four captures (three run at once) give the same bright-pixel counts frame for
  frame.
* Sonic, test map (`BF_TEST_GRENADE_TYPE=Sonic`): goes off at first contact, red tint, the dome
  and ring then the godray shafts, its dark decal after.
* Light (`BF_TEST_GRENADE_TYPE=light`): blue spikes, still burning 8.9 s after; no damage, no
  tint, no decal.
* Hand-over during a throw (`BF_TEST_THROW=1.5,0.07 BF_TEST_SELECT=1,1.1`): the count drops to
  9 at the button, control passes at 1.73 s before the grenade leaves the hand, `BF_GRENADE_LOG`
  prints "throw cut short, grenade back (10 now)", the box shows 10 again and nothing is thrown.
* Sentry (`BF_TEST_GRENADE_TYPE=Sentry`, ten presses 1 s apart): the box counts 10, 9 ... with
  no digit at 1, then shows Frag 10; `BF_TEST_NEXT_GRENADE=12.5,13` steps to Energy, then Gas.
* Roller (`BF_TEST_GRENADE_TYPE=Roller BF_TEST_DETONATE=6 BF_SOUND_LOG=1`): down 0.97 s after
  the press, rolls off ahead; h_19dbc65d at 2.47, 3.47, 4.47, 5.47 s, quieter as it goes
  (0.80 to 0.42); the blast is the Frag's effects.
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
  identical.
* Energy: see **Energy grenade**.

Squad movement and deaths follow the game's own data and the captures:

* Wingman follow (the squad AI's follow personality, decoded from common/ai/personality.xml
  with the event / goal enums from default.xbe): squadmates keep loose places 6-11 m *ahead*
  of the leader (measured from the captures), set off when 3 m from their place and settle
  within 1 m, dash when far behind (or past the data's squad-leash-dist + 10 m), and face the
  leader's heading. Standing a while, Flint, Hawk and Tex kneel (stand2crouch -> rp_crouch_idle,
  crouch2stand to move on); Brutus stays on his feet. They step out of the lane you aim or fire
  down (EVT_SHOT_BLOCKED_BY_FRIEND), sidestep now and then when you shoot them
  (EVT_DAMAGED_BY_PC -> GOAL_DODGE) and dive away from a grenade that lands near them
  (EVT_GRENADE_NEAR -> GOAL_DIVE, the motion scripts' dive clip).
* **Standing on the floor.** The floor is 1.05 m (`GROUND`) under each character's pelvis, but
  the four aren't the same height. So each is raised or lowered by how far the lowest
  skinned vertex of their idle pose is from it (`sole_lift`, up to 0.3 m either way):
  * Tex: raised 0.17 m (his soles went through the floor);
  * Brutus: lowered 0.07 m (floated);
  * Flint: about right (0.01 m);
  * Hawk: lowered 0.15 m (floated).

  That's for standing. A kneel's lowest point (the knee) isn't the standing soles', so each
  crouch clip has its own lift (`clip_lift`): `stand2crouch` and `crouch2stand` through their
  length (16 samples), and `rp_crouch_idle` and the crouch walks as the most over their cycle.
  The model is lifted by the weighted mix of what's playing, so a kneel or a crouch walk
  stands on the floor too. With the standing lift alone, Hawk and Flint knelt in the air. This
  covers the squad's kneels as well.
* **Scopes.** With a weapon that zooms, right mouse toggles its scope (click in, click out;
  other weapons still aim while it's held). Flint steps in twice: in, closer (the weapon's zoom
  doubled: the L-Shot-50's 5x, then 10x), out. The doubling is a choice; the data has one zoom
  per weapon. Switching weapons or reloading drops out of the scope. `BF_TEST_SCOPE=<step>`
  holds a step. A scoped shot leaves from the eye along the crosshair's ray (from the
  muzzle, a little lower, it would land low through the zoom). In the scope the camera
  moves to the character's eyes: the characters' `offset-snipe` camera offset is 0.001 0 0,
  against 3.2–3.4 m behind for walking, running and ready. The field of view narrows by the
  weapon's zoom (`h_efce7f77` on its definition: MK-ASLT 2, Kman Auto 2, Psionic Blast 2,
  Foley 356 Tact 3, L-Shot-50 5, L-Shot-75 10; 0 on the rest), the model is hidden, and mouse
  look slows by the same factor. Shots still go through the crosshair. While scoped, the
  character's own breathing loops: each character's `snipe-sound` (`h_12501487`, beside its
  radar-range), a 2D sound of the common bank (Brutus `h_01a51dd2` 2.9 s, Flint `h_15545426`
  0.7 s, Hawk `h_1ca2a830` 1.7 s, Tex `h_0ce9616b` 1.8 s), with 0.6 s between breaths.
  `BF_TEST_AIM=1` holds aim.

  Two kinds of scope view, as in a capture:

  | Who | View |
  |---|---|
  | Flint (the sniper) | Fills the screen, with a cyan-teal glow at the edges. Her aim is dead steady (she's a synthetic). The game smears those edges with an effect, so the glow here is an approximation. |
  | The others | The view sits inside the game's scope frame `h_1807ec04`: a 256 × 128 quarter (rounded corner, tick marks) mirrored four ways. It's sized so the rim spans 77% of the width, as in the capture. The texture's grey outside is drawn darkened. |

  The others' aim wanders while scoped: a slow drift of up to about 0.7° that moves the view
  and the shots together. In the scope everyone can still move, but only at a walk. The
  crosshair moves to the middle of the screen and grows from 40 to 72 units. The eye stays
  0.25 m off whatever is in front (by the collision), so a close wall can't be looked through.

  Going in and out plays the character's two scope sounds. They're stored beside its
  snipe-sound: `h_00dd1fe0` and `h_02c94d34`, about 0.3 s each, taken as in / out. Flint has
  her own pair (`h_1fd08ea5`, `h_03e0936d`); the others share `h_135ec2ce`, `h_f9ffc972`.
* **Squad hand-over.** It plays one sound as the camera cuts: `h_07949f0d` (common bank,
  0.18 s), found in a capture at all five hand-overs. It replaced the weapon-switch pair used
  before. The name is drawn 1.6x the capture's size, pops in from nearly double that, and at
  the cut swells to 6x while it fades over 0.4 s; the game's swell is about 3x, exaggerated
  here as asked. `BF_TEST_SELECT=<character>[,<s>]` picks that member at 1 s (or s; control passes 0.6 s later), also together with `BF_TEST_GOTO`.
* **The squad's AI table.** Squadmates follow you as wingmen. Their personality
  (`common/ai/personality.xmb`; each character's own `personality-wingman` is empty and
  inherits `h_ea20a17d`) maps events to goals, with priorities and weights. Its events and goals
  are numbers. The names come from default.xbe's name tables: events at 0x3bdc14 (`EVT_DEAD` =
  10, … 96), goals at 0x3bce78 (`GOAL_DODGE` = 59, … 134). Without enemies, these entries
  apply:

  | Event | Goals (weight) | Here |
  |---|---|---|
  | `EVT_GRENADE_NEAR` (pri 28) | `GOAL_DIVE` | dive away from a grenade that lands near |
  | `EVT_SHOT_AT` (20.15) | `GOAL_DODGE` 1, `GOAL_NOOP` 7 | a shot of yours passing within 1.5 m (the distance is a guess): sidestep one time in eight |
  | `EVT_DAMAGED_BY_PC` (15) | `GOAL_DODGE` 1, `GOAL_NOOP` 2 | shot by you: sidestep one time in three |
  | `EVT_BASE` (1) | `GOAL_WINGMAN_FOLLOW` | the formation |
  | `EVT_IDLE_VARY_DATA` | `IDLE_VARY_DATA_PAUSE_CROUCH` 15, `…_PAUSE_CAUTIOUS` 12 (equal weights) | holding position: the first pause kneels (as in the captures), then kneeling and cautious pauses (standing, weapon ready, looking left and right) are drawn, lasting 15 and 12 s |

  `BF_AI_LOG=1` prints their idle choices and dodges.
* Hits come from the game's effect library (common/effects-common.ale, read with
  `../ale_tool.py`) and the flesh world material (21, WMAT_FLESH_HUMAN). Every hit plays its
  debris bundle: blood_puff (smokecard.tga mist), bloodsplat_s (diffuse_big.tga droplets in a
  jet, under a weak gravity field) and giblet (a blood.tga spatter). Each ammo type adds its own
  hit effect and sound: ballistic sparks (Flint, Hawk), the cutter's long white streaks with a
  pale blue flash light (Brutus) and the laser's orange embers with an orange flash (Tex). The
  emitter / appearance numbers in `play_fx.rs` are those nodes' parameters, read as Freelancer's
  ALE ones (Brute Force hashes their names). Each hit also leaves the character's hit decal on
  the ground past them (h_14410af1: the game's white splat textures tinted 128,34,34,220; about
  1 m across, as in the captures) for 40 s, fading over the last 20. Flint bleeds grey: she's
  taken to be synthetic flesh (world material 24, WMAT_FLESH_SYNTHETIC: an inference from her
  grey decals, the data doesn't name a character's material), whose debris bundle h_1f7ca034
  is blood_puff_b, bloodsplat_b and giblet_b - the same nodes in grey and blue-grey, slower
  and smaller - and whose hit sounds for the heavier ammo are metallic. Her hit decal
  (h_02d017b7) is tinted 50,50,50. Verified with `BF_CHARACTER=1 BF_TEST_SUICIDE=1 -- --test`
  captures against Brutus (`BF_CHARACTER=0`): grey mist and splats for her, red for him. A
  hard hit (15+ damage: L-Shot, Bower, a close grenade) can knock a character down: the body
  goes limp (ragdoll, pushed harder at the top so it topples) for 0.7 s, thuds on the ground
  (the surface's landing sound) and gets back up over 0.45 s where it lies - the game uses
  Ipion physics for this (its constraint messages are in default.xbe, with MT_KNOCKDOWN).
* Death: the death cry (chatter e1213623 - the captures' deaths match its lines), a thud, a
  limp body that stays on a blood pool (the character's other decal, h_16d327fd), blood
  splashed round where they fall (the hit's blood effects sprayed up, and five of their hit
  splats on the ground within 1.4 m: the demo's choice, the game's own death blood isn't
  known; Flint's decals are dark grey, 50,50,50, in the data: she's a synthetic), the DNA
  beside where they fell, and - half the time, as the data says - a surviving squadmate's
  answer from the dead member's response tag ("Brutus is down!", "They've taken out Hawk.").
  The DNA is effect h_ee11d51f, the one on the game's DNA canister pickups: powerup_pill (green
  small-flare.tga sprites, added to the picture) and light_powerup_pill, a blue point light
  ("light_" effects are lights). The captures show it as a soft green square: the data's
  texture is the round flare, so the square is how the emulator draws those sprites (its
  explosion sprites are boxy too). On the radar the portrait becomes the member's own skull (the
  characters' h_00c51907 icon) and the member's tab and health bar go.
* **Death camera** (`play_deathcam.rs`, from the reference recording `Friendly Fire 2 + Death
  Cam.mp4`, 28.3-32.4 s). On the frame the character you control dies, the view cuts (no
  blend) to a camera on the body and the reticle goes. The camera uses the character type's
  `offset-dead`: camera block `h_0d41e5f1`, element f13fb0c4, `3.5 5 0` for the squad. The
  objecttypes parser `FUN_0018fa80` stores the seven offsets at +0x1b0 in the character type,
  and all seven are now read into `Game::character_camera`. The camera sits 3.5 m above the
  ground under the pelvis and 5 m back. It aims 11 degrees above and 6 degrees left of the
  pelvis, so the body sits low and right of centre, at about (380, 350) of 640 x 480 as in
  the recording (both angles measured). It circles the body counter-clockwise seen from above,
  travelling to its own right, at a steady 60 degrees per second. It starts from the follow
  camera's heading. The rate is an estimate: 55-66 degrees per second measured through the
  recording's gas smoke.

  Walls: the game's own camera collision here isn't recorded, so this is the demo's choice.
  The camera sits on the line of sight from the body (0.3 m above the pelvis) to where
  `offset-dead` puts it, and keeps 0.3 m in front of whatever the level has on that line, so
  rock never hides the body. It eases in ahead of a wall, looking 20 degrees along the orbit.
  It never comes closer than 2.5 m: where a wall leaves less room, that part of the circle is
  skipped with a cut ahead to the next heading with room.

  On the death frame a squad switch picked just before is dropped, so the death camera always
  runs its full 4 s. A scope ends at once (no zoom easing out, the body not hidden).

  At 3.5 s the next living member's name appears over it, at its final size from its first
  frame, as in the recording (f1910 and f1911 are the same). A manual switch still pops it in.
  At exactly 4.0 s the squad switch cuts to that member's follow camera, and the name swells
  and fades across the cut. Keys 1-4 do nothing while you are dead. With nobody left to take
  over (the whole squad down (#41), or deathmatch) the follow camera stays on the body as
  before.

  The hand-over, for a death or a manual switch, cuts to the camera straight behind the new
  character's facing, the whole body in view, as the recording's cut to Hawk does. If a wall
  is there, it swings in 10 degree steps to the nearest heading either side with room;
  before, the follow camera's clamp could wedge it inside the body. The new character's aim,
  fire, walking and squad-AI idling (the leader's heading it turned to, kneeling) are dropped,
  so the cut is to the plain follow camera with them standing still. Standing a kneeling
  member up is the demo's guess.

  After the cut the follow camera is unchanged: it stays on the heading the cut chose until
  you turn it. If you turn it toward a close wall, its clamp still pulls it in, as close as
  0.2 m. That behaviour is left for #89.

  Not done: the HUD's red tint and fade, the red score popup, steering with the right stick.
  `BF_DEATHCAM_LOG=1` prints the camera every frame (heading, how much was skipped at walls,
  distance and room). `BF_TEST_DIE=<s>` kills the player once per level.

  Verified at 15 fps on sdm_e34 (`BF_MAP=sdm_e34 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=<n>`, plus):
  - `BF_TEST_GOTO=15.3,37,15.0,29 BF_TEST_DIE=1`:
    - frame 15 has the reticle on;
    - frame 16: Tex dies, the view cuts and the reticle goes;
    - frames 16-75: 60 frames turning 4 degrees each;
    - frame 69: "BRUTUS" (t = 3.53 s);
    - frame 76: the cut to Brutus, from behind and whole. After that the test hook's autopilot
      turns the camera toward its goal.
  - `BF_TEST_GOTO=16,37,16,37 BF_TEST_DIE=0.5`, against the cliff:
    - the body stays in view at 5.5-5.9 m;
    - at t = 3.6 s (frame 62) the orbit skips 130 degrees of cliff and lands 2.8 m out;
    - frame 68: the cut to Brutus from behind.
  - `BF_TEST_DIE=1` from the map's start: the cut at frame 76 is to Brutus's back, and he
    stays facing away.
  - `BF_CHARACTER=1 BF_TEST_SCOPE=1 BF_TEST_DIE=2`: Flint scoped through frame 29; frame 30
    cuts to the unzoomed death camera with the body in view.
  - `BF_TEST_SELECT=0 BF_TEST_DIE=1.2`: the manual label shows at frames 16-17, Tex dies at
    18, and the label goes. The death camera runs its 60 frames. "BRUTUS" is the same size at
    frames 71, 72 and 73, and the cut is at frame 78.
  - `BF_TEST_SELECT=0`: the manual switch still pops in (frame 16), and frame 25 cuts to
    Brutus from behind.
  - `BF_TEST_GOTO=16.5,37,16.5,37 BF_TEST_DIE=0.7`, a cliff behind Brutus at the hand-over:
    - frame 71: the cut, from behind with his whole body in view;
    - from frame 72 the hook's autopilot (a `BF_TEST_GOTO` whose goal is its start) sets the
      camera heading to 0 every frame, which points it at the cliff, 1.8 m away;
    - the follow camera's clamp then pulls it in close to him (frames 73-89). That comes from
      the hook and the clamp, not the hand-over.
  - With `BF_DEATHMATCH=1` there is no death camera.
* `BF_COMBAT_LOG=1` prints hits, knockdowns and dives.

Squad: the four squad members are all in play, each with their own health (the characters'
combat-target hitpoints: Tex 115, Brutus 105, Flint 90, Hawk 65) shown on the radar's diagonal
edges. Each bar fills that diagonal's dark channel in the radar frame texture (the frame's own
texels darker than 27/255), starting by the member's portrait, and drains toward it as in the
game. For whoever you control, health also shows in the top-left bar. Grenade blasts hurt everyone in range (the explosion's Damage max, falling
off to nothing at its radius; the thrower takes a flat share, see **Grenades**) and the hurt give a pain grunt (chatter e856009f). Friendly
fire: your shots hit squadmates in the way (the weapon's damage; they say "I'm hit" or "Careful!"
/ "Stop shooting at me!"), and the crosshair turns green while it is on one. At 0 health a
character dies: death cry (chatter ef32191d), the body goes limp as a ragdoll (verlet point
masses on the skeleton, pushed by the hit or blast; its limb bones are 0.09 m spheres that don't pass through each other, and every bone lies on the level's own floors, kept as far above them as the body is thick round it (`flesh`: of the skin each bone moves most, the distance from the bone's line that 80% of it is within, 0.04-0.2 m; a flat 5 cm let a thigh or the chest sink into the floor), and is kept out of its walls - `BF_RAGDOLL_LOG=1` prints the closest limb pair and the lowest bone; the pose takes only the bones' turns from the simulation and keeps their rest offsets, so limbs don't stretch or twist; the ragdoll starts where the body is drawn, sole lift included (see "Standing on the floor"); the torso moves as two solid pieces (hips; chest with shoulders), stiffly joined at the waist, with the head held to the chest; knees and elbows (found by their Bip01 names) bend but don't fold past about 120 degrees or straighten past straight, and only one way: each joint keeps to its side of the line from the upper bone to the end, in the torso's own frame (hips for knees, chest for elbows), the side it was bent at death; a nearly straight limb uses knees forward, elbows back (forward from the toes); bodies keep 98.5% of their speed per substep, lose 60% of their sliding on the ground, and rest once still - but not on their side: a body come to rest propped on a shoulder and a hip (its chest's left-right axis more than 0.6 upright) is rolled on, its upper shoulder pushed 2 m/s toward its back (or front, if it leans that way), up to 3 times (the demo's choice). Shots hitting a body on the ground (dead or knocked down) push it: the first bone the shot passes within its thickness of is brought up to 3 m/s along the shot, and bones within 0.6 m less, so it jolts and rolls (a burst doesn't add up past that speed, so it doesn't drive the body across the ground); it wakes the body to settle again and it bleeds where it's hit, as the living do (blood effects, hit sound, a splat on the ground) (`BF_RAGDOLL_LOG` prints each body hit and the pelvis's position); `BF_TEST_DIE=<s>` drops the player dead at that time, once; `BF_TEST_DIE=1` at 1 s). Both their guns drop, the one in hand and the one on their back (the ragdoll can't feel a stowed gun, so it propped bodies on their side): each falls from where it was, thrown by half of what killed them plus a 1.5 m/s hop, and tumbles as a loose object (see "Pickups are loose"). Then the portrait greys out and the blip leaves
the radar; if you die, the death camera circles your body and control passes to the next
living member after 4 s (see **Death camera** below; `BF_TEST_KILL=<character>` makes the
player shoot that member). The inventory is shared: the
grenades go with control. Every portrait's tab shows the follow-order arrows; a speech icon
appears beside the portrait while that member talks. The one you control is the player; the other
three are AI that keep a formation on your flanks and a little ahead (running / sprinting to
catch up, walking the last bit), look where you look, and
reload on their own. They never fire just because you do - only at enemies they can see (none in the demo yet). Collision: everyone is a 0.4 m circle, pushed out of the pillars and apart from each other. Their footsteps and shots are heard quieter with distance, and they show as
yellow blips on the radar (turning with the camera, 40 m range). `BF_NO_SQUAD=1` (or `BF_DEATHMATCH=1`) plays deathmatch: alone, without the radar.

HUD health / stamina frame: the original's own art, the tutorial's picture of it (`h_17f34cbe`,
256 × 64, only in tutorial.tgz: loaded for it directly). It's cut in pieces: the + and bolt icons
(drawn solid), the frame's left cap, a middle that stretches and the right cap (drawn as glass, 45%:
Bevy blends see-through UI in linear light, so that's what matches a capture's body colour,
22 46 86, ours 17 51 91). Its baked bars are painted its own empty-bar navy (0 20 107) and the live
bars drawn over them (rows 22–27 and 35–40, from column 56). The bars' length follows the
character's maximum health (`combat-target hitpoints`: Tex 115, Brutus 105, Flint 90, Hawk 65;
`BF_DUMP_HITPOINTS=1` lists them): Tex's 184, which gives the capture's frame proportions (5.4 to
1), the others in proportion; the frame's middle stretches with them, and a squad switch resizes
it. Without the tutorial's picture, the frame below is used.

HUD textures found for the panels: the health / energy frame is built from its corner
`h_0cdf5876` (32 × 16 end cap, mirrored four ways; it holds the step from the raised ends down to
the middle, whose level its inner column carries along), drawn as glass at 55% over a faint
tint. Its two channels are open in texel rows 6–11 and (mirrored) 20–25, columns 5–174; each bar
fills its channel exactly, and its icon is level with it. The bar fills (`h_e801997b` red,
`h_e11c7d04` blue, 4 × 8) are coloured in 6 of their 8 rows, matching the channels' 6; only those
rows are drawn. (Placed by hand, the bars sat over their channels' lines and left gaps at both
ends; drawn whole, the clear rows put the colour off-centre.) The grenade's panel is
`h_10891a71` (64 × 32, glossy, cut top-left corner), 9-sliced. Neither is referenced in the
data; both were found by shape. (The tutorial's `h_17f34cbe` is only a baked picture of the
whole bar.) Radar health channels are filled as bands (distance from the centre and extent
along the diagonal, from the dark texels found), so the channels' highlight lines no longer
leave gaps.

HUD (`src/bin/play_hud.rs`), as in the game: health / energy top left, weapon panel top right
(the weapon's own icon from its definition, clip / reserve; every weapon listed with names for
2.5 s after a switch, the held one in orange), radar with the squad's four portraits round it (Tex top, Hawk left, Flint right, Brutus bottom),
grenades bottom right, and the held weapon's own crosshair (its definition's `reticule-prefix` texture, e.g. f4ea7a92 for the MK-ASLT) above centre (y 187 of 480) - aiming and shots go
through it. Layout is in the game's 640 x 480 screen (measured from an xemu capture) inside a
centred 4:3 area; textures are the game's own from `common.tgz` (stored upside down, drawn
flipped). Health and energy are static: the demo has no damage. Ammo is a
full clip plus ten in reserve per weapon. `BF_NO_HUD=1` hides it.

How it works (`src/bf/locomotion.rs`): animation names are hashes, so clips are chosen from
their data: root-channel velocity (forward is -Z; sprints are fastest), clean looping, head
height (rejects crouched / kneeling variants) and arm span (rejects arms-out reference poses).
The squad's only sideways clips are dodge rolls, so aimed movement is legs-run plus an
upper-body twist spread over the spine and head (`aim_chain` / `twist`). Movement is root
motion: each frame the character moves by exactly what the playing clips' root channels move,
crossfaded over 0.2 s, so speeds and footfalls match the original animations.
Jumping uses the game's named clips (`Sc_w1_jump_crouch/launch/fall/land`, `Sc_4leg_w1_jump_*`;
names recovered from the motion scripts) with simple gravity; a jump pressed during a dodge is
buffered for 0.4 s.

Weapons (`src/bf/weapon.rs`): every character carries their real starting inventory
(objecttypes: Brutus Feral Cutter + Bower 20, Flint L-Shot-50 + Confed LZR-10, Hawk Foley 356
Tact + Confed LZR-23, Tex Confed LZR-23 + RVG50 Minigun), with names from the English string
table, models from the level's weapon archetypes (24-byte static vertices; the minigun is a
compound whose barrel spins), and the definition's fire rate, fire sounds and range. A held gun
hangs from the trigger hand: the character's hand hardpoint lined up with the weapon's trigger
grip hardpoint (both point + w-x-y-z quaternion). That pairing makes every squad weapon point
straight ahead and level in the game's ready-pose clips, and puts the support hand on the
foregrip in the carry clips. Locomotion follows the weapon's stance. Each character's two motion
sets, `Sc_w1_*` and `Sc_w2_*` (idle, walk, run, dash, backpedal, reload, throw), belong to the
weapon classes of its two slots (`<limit-weapon one="CLASS n" two="CLASS n">`, class = the
weapon's `weapon-type`: 0 light, 1 medium, 2 heavy):

| Character | `Sc_w1_*` | `Sc_w2_*` |
|---|---|---|
| Brutus | medium | heavy |
| Flint | medium | light |
| Hawk | light | medium |
| Tex | heavy | heavy |

A gun takes the set of its class, else that of its slot. So Brutus holds his medium Bower 20 in
his medium stance, like the Feral Cutter, though it's his second weapon. Arms then carry each gun
the way the game animates it (Tex's minigun low in both hands, Flint's LZR-10 one-handed), and
aiming or firing switches to that set's `rp_*` ready-pose clips. The barrel is then turned onto the crosshair (yaw and pitch, two
passes, over the spine bones that carry the arm), so shots leave along the barrel (within 1
degree). The other weapon is stowed for its inventory slot: the weapon's holster hardpoint (`h_10ab40ac`,
on every gun) on the character's stow hardpoint for that slot, which puts Tex's guns vertical on
his ramps, Brutus's diagonally across his back, and Flint's and Hawk's sidearms barrel-down on
the hip. Switching plays the character's own weapon-change overlay from its motion script (Flint and
Hawk `Sc_w1_2_w2` / `Sc_w2_2_w1`, Brutus `Sc_w1_2_w2rifle` / `Sc_w2rifle_2_w1`, Tex
`Sc_wc_rifle2cannon` / `Sc_wc_cannon2rifle`, whose ramps swing the gun over his shoulder) on the
upper body over whatever the legs are doing. The clips carry timed events (an event channel of
f32 time + name hash per event): at `drop_weapon` the held gun goes to its stow point, at the
grab event the other one comes to the hand, each with the weapon's handling (`pickup-sound`)
sound; no firing or aiming until the clip ends. Shots: muzzle flash and light,
recoil, tracers that fly to what the crosshair is on (ground or pillars) and a spark there.

Sounds (`src/bf/audio.rs`) come straight from the game's banks: `sounds-<level>.xmb` maps sound
ids to Xbox ADPCM data in `sounds-<level>.mem`, decoded to 22 kHz WAV in memory. Each character's
archetype names its jump grunt and footstep type (Tex 1, Flint 2, Brutus 3, Hawk 4), and
`world-materials.xmb` gives every surface four footstep variants per type plus landing and slide
sounds. Footsteps fire when a toe comes down into the lowest quarter of its swing in the clip
being played. Surface sounds live in level archives: `BF_LEVEL=<name>` (default `e01`) picks the
level; `mp1`'s bank is merged in too, since a mission only carries its own squad's footsteps.

Test hooks: `BF_AUTOPILOT=1` plays a scripted run (idle, run, turn, sprint, aimed walk,
backpedal, dodge, standing jump, running jump, aimed fire, weapon switch, firing on the run); `BF_TEST_EXPLOSION=1` sets off the selected grenade type 6 m ahead every 1.5 s; with `BF_TEST_GOTO`, `BF_TEST_THROW=<s>[,<hold s>,...]` holds the grenade button (G) at those times (see **Grenades** for the rest of its hooks); `BF_ANIM_PROBE=<script>,...` prints each character's clips by script name (duration, root motion over a cycle, events); `BF_DUMP_TEXTURE=<file>:<hex id>` writes a decoded texture (raw RGBA after a u32 width and height). `BF_AUTOPILOT_SPIN=<rad/s>` makes it look down and turn the camera during the standing fire; with `BF_CAPTURE=<dir>` it saves every frame at a
fixed 15 fps. `BF_MUTE=1` silences audio, `BF_SOUND_LOG=1` prints every sound event with its time
and action, `BF_DUMP_SOUND_IDS=<dir>:<id,id..|all>` decodes sound ids to WAV, `BF_DUMP_SOUNDS=<dir>` writes the character's jump and surface sounds as WAV files
and exits. `BF_DUMP_WEAPONS=1` lists every character's weapons as loaded (definition, model parts,
hardpoints, fire sounds) and exits; `BF_SHOT_LOG=1` prints each shot's barrel and shot
directions. Camera hooks: `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` / `BF_CAMERA_DISTANCE` set the
start (with `BF_TEST_GOTO`, `BF_CAMERA_PITCH` holds the pitch throughout, e.g. to fire down at
something), `BF_VIEW_YAW` turns only the rendered view (aim unchanged), `BF_START_WEAPON=1` starts with
the second weapon. Captures wait 60 frames for shaders to compile before the clock starts.

`bf_viewer` decodes everything itself (ports of the Python tools in the repo root):

| Module | Python original | What |
|---|---|---|
| `src/bf/hash.rs` | `xmb_tool.name_hash` | name hash (CRC32 variant, game's own table) |
| `src/bf/archive.rs` | `tarfile` | level `.tgz` archives |
| `src/bf/bxml.rs` | `xmb_tool.py` | BXML documents + BXSD schemas (LZ token stream, typed values) |
| `src/bf/texture.rs` | `tex_tool.py` | texture index + DXT1/3/5 and swizzled formats -> RGBA |
| `src/bf/character.rs` | `char_render.py` | skeleton, skinned meshes (incl. NV2A push buffers), skin materials, animation channels |

It loads `common.tgz` (all BXML decoded against the schemas) in about 0.3 s. Squad skins
live in the level archives, which are opened on demand for their textures only.

Check against the Python tools:

```
cargo run -- --dump brutus 20 0.5     # counts, bind check, bone positions for anim 20 at 0.5 s
```

All four characters match `char_render.py` exactly (bone counts, triangles, animations) and
bone positions agree to ~1e-6.

## Controls

| Key | Action |
|---|---|
| 1-4 / Tab | switch character (Brutus, Flint, Hawk, Tex) |
| Right / Left | next / previous animation |
| PageDown / PageUp | jump 10 animations |
| Q / E | previous / next facial pose (default: neutral) |
| L | toggle game shading vs the old flat look |
| Space | pause / resume |
| `[` / `]` | half / double speed |
| Left-drag, wheel | orbit, zoom |
| F | reset camera |
| G | toggle ground plane |

Animation numbers match `python char_render.py anims <name>`.

## Notes

* The camera follows the skeleton root, so animations with root motion stay in frame. Skinned
  meshes have frustum culling disabled (their bounding box is the bind pose's).
* Shading: skins are `BF_CS_rt` ("Color-Specular") materials whose texture alpha is a specular
  mask. The skin doubles as Bevy's `specular_texture` (alpha -> reflectance, the
  KHR_materials_specular convention; needs the `pbr_specular_textures` feature), a generated
  roughness map makes masked areas glossy, and the material's constants set tint / strength /
  gloss (read as h_16e7f952 colour, h_e1664805 level, h_e7604658 glossiness: inferred from the
  values, the names are hashed). Lighting is a warm key with shadows, cool fill and a rim light
  over a dim ambient. `BF_FLAT=1` (or L) gives the previous flat look for comparison.
* Faces: human faces are modelled mid-speech, so a clip from the `<name>face` set (lip-sync /
  expressions) is always layered on the face bones; the default is the neutral first clip.
  Q/E cycles through all of them (and "raw bind").
* Test hooks: `BF_CLOSEUP=1` frames the head. `BF_SCREENSHOT=out.png` (plus optional `BF_CHARACTER=0-3`, `BF_ANIM=<n>`) plays for
  2 seconds, saves a screenshot and exits. `BF_CHARACTER` also sets the starting character.
  `BF_DATA_DIR` overrides where the `.tgz` archives are read from. `BF_FRAMES=<n>` captures n
  frames (`out_000.png` ...) at a fixed 15 fps from the clip start, for GIFs; `BF_NO_HUD` hides
  the overlay and `BF_CAMERA_DISTANCE` sets the starting camera distance; `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` (radians, yaw relative to
  the front) and `BF_CAMERA_TARGET_Y` aim it, and `BF_FACE=none|<n>` picks the face clip (none = raw bind face).
* First build compiles Bevy (about 7 minutes); later builds take under a minute. The dev profile
  optimises dependencies, so `cargo run` is smooth; `--release` would trigger a second full build.
* Bevy is pinned to 0.16.1 in `Cargo.toml`; Bevy breaks its API each release, so upgrade
  deliberately.
* Toolchain: `stable-x86_64-pc-windows-gnu` plus WinLibs MinGW-w64 (provides the `dlltool`/`as`
  that the bundled MinGW kit lacks). dlltool fails on paths with spaces (like "XBE Mod"): if
  the project's path has one, send the build output elsewhere in a local, uncommitted
  `viewer/.cargo/config.toml`, e.g. `[build] target-dir = "C:/Users/<you>/.cargo/target/bf_viewer"`.

## Level viewer (`bf_level`)

`cargo run --release --bin bf_level -- [level]` draws a level from the original archives:
terrain, every placed object (game objects, pickups) at its transform, the sky, and the level's
fog. The default is `sdm_e34`, the squad deathmatch map "Battle of Bulgar" (SDM Level 5 in
campaign-bf.xml). Right-drag looks around, W/A/S/D/Q/E fly (Shift for faster), 1-9 jump to the
level's own flyby cameras and 0 to an overview, F toggles fog, P prints the view. Test hooks:
`BF_LEVEL_CAMERA=<n>`, `BF_VIEW=x,y,z,yaw,pitch`, `BF_NO_FOG=1`, `BF_SCREENSHOT=<file.png>`,
`BF_LEVEL_DUMP=1` (placements, untextured materials, idle effects), `BF_PART_DUMP=<hex
archetype>` (each part's geosets: offset, bounds, normals, winding, UV direction; then exits).

* Terrain vertices (type h_ef44f398) carry no float position. Slot 8 of the game's vertex
  format table gives the layout: SHORT2, packed normal, two float2 UVs, NORMSHORT3, 2 bytes.
  The terrain vertex shader (default.xbe 0x3d50f8, disassembled with the NV2A instruction
  layout) decodes the position:
  * |a| = row × 65 + column on a 2 m grid;
  * the signs of a and b give x and z;
  * |b| is the height in 1/16 m.

  Those scales come from run-time shader constants. I fitted them to the data:
  * the second UV set is exactly the grid ÷ 10;
  * heights land on the level's 26–54 m range;
  * placed objects sit on the ground.

  Terrain triangles wind the other way from objects'.
* Terrain layers: one geoset per texture. Layers after the first are blended in by their
  material's "alpha" mask. Each mask is a single 128 px image over the 256 m terrain (x and z
  from −128 to 128, v toward −z), the orientation that best matches where each layer's
  triangles are.
* A level `<transform>` is a 3 × 3 rotation followed by the position. The rotation's columns are
  the object's axes; read that way, the level's cameras come out level and looking down −z.
* Object types map to mesh archetypes through objecttypes (`mesh-name` beside the type's
  `<base name>`). Material types h_02dca948 and h_1c5f7aab are alpha-tested (grass, fences,
  grates, signs, cables); the rest use texture alpha for shine.
* Material type h_0e7658e4 is the glow shader: a colour texture (its alpha a shine mask) plus a
  glow texture (`h_e01baa40`) drawn as emissive. That's what lights the ceiling light strips in
  Bulgar's buildings and the lava cracks in sdm_m07's and sdm_e13's rock. It used to be treated
  as alpha-tested, which cut the rock into floating dark flakes.

### Map menu

`cargo run --bin bf_play` with no `BF_MAP` opens the front end, rebuilt after the game's own (the
capture in `todo/`). It's laid out on the game's 640 × 480 screen and scaled to the window.

* **Title.** START (DEMOS is shown but not here).
* **Main menu.** CAMPAIGN, DEATHMATCH, SQUAD DEATHMATCH, OPTIONS, CREDITS, with the selected
  word white over a faint, larger echo, and its description beside the logo. Only DEATHMATCH
  and SQUAD DEATHMATCH lead on; the game's mode panel (split-screen / System Link) is skipped.
* **SELECT MISSION.** A carousel of the mode's maps (previous, current outlined, next) with the
  map's name, planet and description. The level list (`common/campaign-bf.xmb`) gives each map
  its kind (`Type`): `h_120785ef` deathmatch (the 7 arenas, mp*), `h_efb18d28` squad deathmatch
  (the 7 sdm_* maps); missions are `h_17506391`.
* **Playing.** One window throughout. bf_play's app has three states: `Menu`, `Loading` and
  `Playing`. Choosing a map shows the LOADING screen while a thread loads the game data, the
  level and its collision; the map then plays in the same window. Deathmatch plays alone, without
  music and without the radar (the squad's portraits, blips and health channels), as in a capture. In the
  game, Backspace removes everything the map spawned (whatever wasn't there when it began),
  and the menu comes back on SELECT MISSION with that map selected. Each map installs its own
  collision (`arena::Arena::install`, replacing the last one).
* **Controls.** Arrows / WASD / d-pad move, Enter / Space / A select, Esc / Backspace / B back
  (Esc at the title quits). The mouse points and clicks.

All of it is the game's. The front end itself is defined by `common/game-options-en.xmb`: its
menus, item positions, menu-style colours and fonts, music player and `global-sounds` (menu
event -> sound). The menu code in default.xbe sends the events: 0x1038e0, a control
stepping its own value by one (the map carousel scrolling), sends `h_ebf26601` (-> `h_1dffc5a1`),
`h_f06bc0ab` (A selected) or `menu_error`; 0x100250, the focus moving to the previous / next item
(through 0xffb60's neighbours), sends `h_e29fd995` (-> `h_ed2b5e99`); a menu opening sends
`h_f97ceaf0`. (`media/Sound.xsb` +
`Wave.xwb`, with cues like DownloadComplete and CardPut, belong to the content-download UI.)

| What | From |
|---|---|
| Words and descriptions | the string table (`SELECT MISSION` h_e7b4fd5a, `Planet: %s` h_fa5634c8, …) |
| Heading font | atlas h_e0afcd52 (the styles' `heading` font; rows in ASCII order) |
| Body font | atlas h_e4e4d2f4 (glyphs from `!` on in reading order, two sizes; the first is used): `!` to `~`, boxes for the codes without a glyph, then Latin-1's `¡` to `ÿ` (counted back from the last) |
| Copyright | the title's static text `h_e3603ae4` at (50, 418), 542 × 20, centred, style h_e76e1577: 153 209 251 at alpha 200, black outline, body font 12 pt |
| Button icons | atlas h_0b630034 (A, B, X, Y, L, R row) |
| Colours | game-options' menu-style: words normal 30 110 150, selected 215 236 251; descriptions 153 209 251; every style's outline (`h_ed70ff4f`) black, drawn as a 1-unit outline plus drop shadow |
| SELECT MISSION | the panel h_e462c760 (7 splash_screen textures at alpha 180, at (66, 50)); the carousel's arrow h_158f87c1 stretched to 20 × 128 (the right one mirrored); title 153 209 251 in the heading font, no outline, typed; map name 153 209 251 (body font 16 pt); recommended players (dm-data `h_0398b8bf` min/max, `Recommended for %d - %d players.`; none on the squad deathmatch maps), planet and description 148 198 149 (14 pt) |
| Loading screen | game-options' LOADING menu (`h_f6551df5`): a 256 × 256 ring at (192, 112) on black, four mirrored quarters of `h_06477be9` or `h_1ec0f086` (the ring with its big blocks on the diagonals / on the axes), shown in turn every 0.1 s (a capture), "LOADING" (`h_0f28682d`) in the heading font, no movie or music. Shown while the map loads |
| Layout | game-options' item positions: words right-aligned in boxes ending at x 279; logo at its own 512 × 256 (title (194, 106), main (194, 18)); description box (312, 240) |
| Animations | translation-anim: items slide in from y 240 (0.25 s); color-anim: fade in (0.5 s); select-anim: the selected word's echo grows (from the word's middle) to 1.7× and fades from alpha 150 in 50 185 250 (0.5 s), then pulsing every 0.3 s while the word stays selected or hovered; the logo slides up from the title's place into the main menu; panel titles type out (0.125 s a letter); their sounds `h_e8ace091` / `h_e1abd007` (splash_screen) |
| Logo, title bar | splash_screen textures h_f1e0ac39, h_190a4ea7 |
| Previews | the level list's `texture-name` (splash_screen) |
| Music | `menu_dub1` in `ml-sounds/en/splash_screen-en.tgz` (the music player's sound `h_fa71a2f8`; `sounds/splash_screen.xwb` is an empty stub) |
| Menu sounds | game-options' `global-sounds` (sounds-common): moving between words `h_ed2b5e99` (heard at both moves in a capture), carousel scrolling `h_1dffc5a1` (the code's value-stepping event, above), select `h_e6b8bf54`, can't `h_e47668bc`, menu opens `h_f3a5b12b`. Going back has no sound of its own: as in a capture, the menu it returns to plays its opening sounds (item slide `h_e8ace091`; the main menu also its logo slide `h_e1abd007`, the logo sliding down from the title's place each time) |
| Background | `data/movies/menuBack.bik` |

Bink can't be decoded here, so the background plays from JPEG frames made once with ffmpeg:

```
ffmpeg -i "Brute Force/data/movies/menuBack.bik" -vf fps=15 -q:v 4 decompiled/movies/menuBack/%04d.jpg
```

(571 frames, 8.6 MB; without them the menu shows on black.)

Starting up, as in a capture: the window opens at once on the boot screen, a dim "B" emblem with
"Loading" over it at the screen's bottom left (44, 364), while the game data is read on a thread.
The emblem pulses every 2 s, measured frame by frame from a capture (against its brightest: 0.29,
down to 0.12, up to 1 and held 0.9 s, back to 0.3), and fades out as the loading ends. Then the opening movies play: the Microsoft Game Studios
logo, the Digital Anvil logo and the intro (`data/movies/MGS_Logo_Final.bik`, `DA_Logo_Final1.bik`,
`Intro_Montage.bik`). Any key, click or pad button skips the one playing; after the last, the
title. The boot screen is drawn by the game before any data is read and isn't in the data files or
the XBE's images (`$$XTIMAGE` is the dashboard's title picture, `$$XSIMAGE` the save icon), so
`decompiled/boot/loading_emblem.png` and `loading_text.png` are cut from a capture (2560 x 1440,
the 640 x 480 screen at 3x, scaled back to 1:1 and averaged over the emblem's brightest frames;
greys as colours, opaque). The movies are converted once. Their sound is in several tracks (as Bink on the Xbox
plays them in surround): 0 front left / right, 1 a mono low channel, 2 the rear pair, and from 3
on the centre, one track per language (the narration with the centre's share of the music: five
on the intro, nearly alike but for the voice; 3 is English: the only one Windows' English speech
recogniser follows, and the Xbox's language order). They are mixed down to stereo, the centre and
rears at -3 dB, the low channel at -6 dB, under a limiter (track 0 alone left out the intro's
narration and most of the MGS logo's sound). The credits movie has one stereo track.

```
mix='[0:a:0]aresample=44100,aformat=channel_layouts=stereo[f];[0:a:3]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[c];[0:a:1]aresample=44100,aformat=channel_layouts=stereo,volume=0.5[e];[0:a:2]aresample=44100,aformat=channel_layouts=stereo,volume=0.707[s];[f][c][e][s]amix=inputs=4:normalize=0,alimiter=limit=0.95:level=0[out]'
for m in MGS_Logo_Final DA_Logo_Final1 Intro_Montage credits; do
  ffmpeg -i "Brute Force/data/movies/$m.bik" -vf fps=30 -q:v 5 decompiled/movies/$m/%04d.jpg
done
for m in MGS_Logo_Final DA_Logo_Final1 Intro_Montage; do
  ffmpeg -i "Brute Force/data/movies/$m.bik" -filter_complex "$mix" -map "[out]" -ac 2 -ar 44100 decompiled/movies/$m/audio.wav
done
ffmpeg -i "Brute Force/data/movies/credits.bik" -map 0:a:0 -ac 2 -ar 44100 decompiled/movies/credits/audio.wav
```

(10 s, 15.7 s, 67 s and the credits 150 s, 163 MB; a movie without frames is left out.) The main
menu's CREDITS plays the credits straight away, their frames read from disk as they're shown (any
key, click or button stops them) and comes back to the main menu on CREDITS. Movies played on their own fit
inside the window, whole (the menu's background covers it). `BF_MENU_SCREEN=credits` picks CREDITS. `BF_NO_INTRO=1` skips the movies;
the menu screenshot hooks skip them too and count frames from the menu, unless `BF_SHOT_EARLY=1`
(frames then count from the start: the boot screen, the movies).

When the menu is skipped: whenever `BF_MAP` or a test hook (`BF_CAPTURE`, `BF_TEST_GOTO`, …) is
set, or with `BF_NO_MENU=1`. Test hooks: `BF_MENU_SHOT=<file.png>` saves the menu and quits;
`BF_MENU_SCREEN=title|main|dm|sdm|loading` with `BF_MENU_PICK=<n>` picks the screen and entry;
`BF_MENU_SHOT_FRAME=<n>` takes the shot at frame n (screenshots step a fixed 1/60 s a frame, to
catch the animations); `BF_MENU_GO=1` plays the picked map (the screenshot hook runs on into it); `BF_TEST_BACK=<n>` goes back to the menu after n frames of a map.

The arenas Chamber (mp2), Ammo Depot (mp3) and Cavern of Fire (sdm_m07) are built from objects
alone, with no terrain mesh. Their rooms are lit by the
level's point lights (see Point lights below).

### Other maps

Every squad deathmatch map loads (`BF_MAP=sdm_e01|sdm_e10|sdm_e13|sdm_e34|sdm_e40|sdm_m03|sdm_m07`,
or `cargo run --bin bf_level -- <map>`). What it took:

* **Terrain grid per level.** The terrain vertex shader splits a vertex's first short into row
  and column by 8 × the terrain's width in blocks + 1, and scales both by the cell size
  (`Terrain h_e023cdc9`: 1, 2 or 4 m). Blocks are 16 cells.

  | Map | Blocks per side | Cell |
  |---|---|---|
  | sdm_e34 | 8 | 2 m |
  | sdm_e01, sdm_e10 | 16 | 1 m |
  | sdm_e13 | 8 | 4 m |
  | sdm_e40 | 16 | 2 m |
  | sdm_m03 | 19 | 2 m |

  The terrain layer masks span the whole grid.
* **Baked terrain light.** Each terrain block carries 16 × 16 cell values (`blocks
  h_e2d79bad`, 0–255): the terrain's baked lighting, cliff shadows included. Terrain is drawn as
  texture × that light. The orientation (blocks row by row, x and z increasing) was found by
  correlating it with the terrain's slope lighting.

  The level's dynamic terrain lights are dim; sdm_e40's key is 0.2, low in the sky. The baked
  light is what the map-select previews show: bright sdm_e40 and sdm_e10, night sdm_e01.
* **Liquid surfaces.** A liquid object's surfaces use one of two shaders, drawn by
  `LiquidMaterial` (`src/liquid.wgsl`):
  * the animated liquid `h_f124a774` (lava, toxic rivers): `texture-0` and `texture-1`, each
    a tile `texture-scale` / `texture-scale-1` of the mesh's uv across (by the sizes that gives:
    sdm_e13's lava, 1.5 km over 15 uv, has 10 m tiles at 0.1; read as repeats, a 256-texel
    texture stretched over a kilometre, blurred, and scrolled tens of m/s), scrolling in tiles
    per second (two unnamed pairs, by their values u and v of each layer: `h_0e41af5b
    h_0e2b9302`, `h_e002ae8e h_fb0bff34`), combined by
    `blend-mode-1` (0: over by its alpha, sdm_e40's lava crust; 1: added, the toxic river's
    green over its dark swirl). Every layered liquid's `blend-mode` is 2, read as the console's
    modulate x 2: the colour doubled. Lava (liquid-type 2) gives all its colour as light, toxic
    (4) 80%; both are solid (the materials' own opacity, 60-200 / 255, is left out: captures of
    Singe's and Cavern of Fire's lava and Shanty Town's acid show them opaque, near yellow).
  * the water `h_0f8904b9`: `texture-0` tiled (`scale-u`, `scale-v`) and scrolling, tinted by
    `color`; over it the reflection by a fresnel ramp (`h_0b85cb09`, a 128 x 1 alpha strip: more
    at a glance). The game's reflection is `texture-1`, a texture no archive holds (rendered at
    run time); here it's the level's sky colour tinted by `color-1`, rippled by the water's own
    texture, 15% looking straight down (without any a pool seen from above was its murky colour
    over a bed of the same colour: it vanished) to 50% at a glance; 92% opaque, its colour doubled
    too (no blend-mode: a choice; a capture of sdm_e10's pond measures 57 60 36, ours 49 49 18).
  * Water ripples: its texture twice (scale-u, scale-v and scale-u-1, scale-v-1, each drifting:
    at least 0.02 tiles/s, the two crossing; sdm_e10's own speeds, 0.0002 and 0.01, left it still;
    the water's `framerate` constants, 17 and 25, hint at an animation of the game's own, not
    traced). Their brightness slopes, as heights, tilt the surface: the light, the sun's glint
    and the reflection ripple, and the water's colour is shaded by the tilt from one side. It
    gives 35% of its colour as its own light: sdm_e10's dim lights alone left the pond near
    black, its ripples lost, where a capture shows it evenly lit.
  * Liquid surfaces have a depth bias: Singe's lake lies centimetres over terrain painted with
    lava veins, and seen low the two fought, the veins showing through the lava.
    `BF_LIQUID_TEX=<hex>` puts one texture on every liquid layer (a test).
  * A liquid's drawn surface is moved onto its collision's top (what's waded in): sdm_e40's
    pools draw theirs 1 m higher, so from the side the water floated over the floor (from above
    it lined up, so it seemed to come and go with the view); lava and the rivers already agree.
    `BF_LIQUID_DEBUG=1` draws every liquid surface solid white.
  Other names found by hash: `framerate`, `framerate-1`, `lifetime`, `blend-mode`.
  `BF_LIQUID_LOG=1` lists a map's liquid surfaces (material, textures, constants).
* **Splashes.** A liquid type also names its effects and sounds (`<h_fa2f5452>` entries:
  h_08f6cd94 a shot striking it, h_e4791abc a small splash, h_07ab4143 a big one, h_f458daca the
  ring round a wader; `env_xl_*` water, `env_sl_*` lava, `env_fl_*` toxic, `env_tl_*` type 3,
  `env_il_*` type 0 with bubbles) and four sounds (unnamed; taken in the effects' order). Going
  into a liquid splashes (big when falling faster than 5 m/s), wading leaves a ring every 0.5 s
  while moving, and a shot through a surface splashes where it goes in.
* **Liquids aren't solid.** A pool (sdm_m03's toxic river, sdm_e13's lava, and pools on sdm_e10,
  sdm_e40, sdm_m07, mp4, mp6 and mp7) is a placed object of class `h_04366a6a` whose collision
  is one flat plane at its surface. That plane is left out of the floor and walls: characters
  wade and sink to the bed. A liquid's type
  (objecttypes `<h_fa2f5452>`, read by 0x193420) gives its kind (`liquid-type`) and three damage
  rates per second (h_06f6a40d, h_eba69d33, h_e0489ea0):

  | liquid-type | Liquid | Damage per second |
  |---|---|---|
  | 1 | water | 0, 0, 0 |
  | 2 | lava | 50, 50, 30 |
  | 3 | (unnamed) | 0, 10, 30 |
  | 4 | toxic | 50, 50, 50 |

  How the game uses these values isn't traced (a capture of sdm_e13 had Brutus wade waist-deep
  through lava for 10 s at full health, then die the moment he went under). Here a harmful
  liquid burns from the first touch: 20% of its highest rate at the surface, rising with depth
  to all of it at 1.6 m (lava: 10 hp/s ankle-deep, about 30 waist-deep), dealt every 0.4 s
  without blood; going under kills. A fall into a liquid doesn't hurt.
* **Terrain hue.** The baked light map gives the ground's brightness. Its hue comes from the
  terrain's own lights (ambient + key + fill, the pair marked `h_e02aba1c`), at 60% strength.
  On Bulgar those lights are blue-grey, and the captures show grey ground and hills, not sand.
  The 60% is calibrated on a capture of the gate: the ground there measures 73 81 79; ours was
  79 74 64 and is now 68 77 74. `BF_DOOR_LOG=1` lists the doors (type, place, leaves); Bulgar's
  big gate is `h_0792d16a` at (-23.2, 74.3).
* **Point lights.** A level's light-object `h_ea460e64` is a point light: a place, a colour
  (`h_f08eb2f3`, 0-1), a `range` (4-40 m) and a `falloff` kind (3 or 4 on all 1290 of them; the
  schema leaves its enum, `h_f30c3302`, unnamed, and the light's setter goes through the generic
  property table, so its meaning isn't traced; the other `falloff`s, explosions', are
  `DFALL_NONE / LINEAR / EXPONENTIAL / HALF_LIFE`). The game lights static geometry per vertex
  by them (the scenery meshes carry no baked colours). Here the level materials do it per pixel
  (`LevelMaterial`: bevy's standard material plus `src/lamps.wgsl`): each lamp's colour x
  (1 - distance / range) x N.L, unshadowed, nothing past the range, every falloff kind taken as
  linear. Bevy's own point lights fall off as 1 / d^2 and couldn't follow that. Characters take
  the lamps' light at their chest from every direction, half of it, as their texture added
  (emissive). Ammo Depot has 24, Cavern of Fire 22, Bulgar's squad map 2, campaign maps up to
  160. `BF_NO_LAMPS=1` leaves them out (to compare); `BF_LIGHT_LOG=1` lists them and each start
  point's nearest.
* **Grass.** Bulgar's grass clumps are see-through cards standing upright with their normals
  lying flat (every normal's y is 0). Lit by those, they caught almost none of the high key
  light and their backs none at all: they were black. Cut-out cards whose normals all lie flat
  (|y| < 0.1) are given normals pointing up, so they're lit like the ground, both faces alike.
  `BF_MAT_LOG=1` lists the materials (type, textures, average colour).
* **Shadows.** The scenery casts no shadow; only characters do. The game lights static
  geometry per vertex by the level's lights (every light-object is `enable-static`), with no
  shadows, so building interiors are lit like the walls outside. With shadow-casting roofs
  they were black, and a jungle canopy darkened the whole floor.
* **Sky.** A level's `<sky><object mesh-name>` is a set of layers, drawn in order. On Bulgar:
  a flat blue top, a panorama band of mountains and clouds, a moon, and a cloud swirl. They use
  the self-lit shader `h_f539fe8c`, whose texture is `h_e01baa40` (the texture lookup missed it,
  so skies were flat grey). See-through layers are blended. The sky moves sideways with the
  camera, so its mountains stay on the horizon.
* **sdm_m07's background.** It has no sky (`h_ee630063="false"`, a cavern), and its terrain
  mesh is missing, so the background shows through where the terrain should be. There, the
  background is the fog's colour rather than its bright blue.
* **sdm_m07.** It names a terrain mesh that's on no disc archive. It loads without terrain: the
  map is all objects.

Known gaps:

* sdm_m07 is much darker than its preview's lava glow.
* Bulgar's start ravine is darker than in the game (the terrain's baked light there is low).
* Bulgar's interiors are much darker than in the game: walls facing away from the level's
  two directional lights get only its dim ambient. Captured mean 40 55 60, ours 23 27 32.
  Static meshes carry no per-vertex lighting (24-byte vertices: position, normal, uv), so the
  game lights them some other way, not yet traced.
* Mission maps (e01, m01_a, …) load their geometry but have no start points; missions place
  the player with scripts.

### Playing on a map

`bf_play` now runs on Battle of Bulgar by default (`BF_MAP=<level>` picks another level,
`BF_MAP=flat` brings back the checkered test floor, `BF_START=<n>` picks a start point,
`BF_TEST_GOTO=x,z,x2,z2[,y]` starts the player at one point (on the floor below height y if
given, e.g. a roof) and runs them toward another, or stands still when both points are the same;
`BF_FIND_STEEP=1` lists slide surfaces steeper than 50° and ledges with a drop of more than 6 m,
then exits; `BF_SLIDE_LOG=1` prints slides, falls and landings; `BF_ALE_LOG=1` prints particle emission;
`BF_NO_IDLE_FX=1` leaves out the power-ups' effects):

* **Spawning.** You start at the level's start point. Points 0–3 are one squad's spawn on the
  hill, 4–7 the other's, and 9 is in the base.
* **Collision.** It uses the game's own collision surfaces (`src/bf/collision.rs`,
  `src/arena.rs`). On sdm_e34 that's 43,348 triangles.
  * Where they come from: `objects-<level>.ipn` holds them in the Ipion physics engine's format.
    The terrain blocks, the 23 invisible blockers and every placed object's parts each name
    theirs. Door leaves have their own and stop blocking while open.
  * Material: every triangle carries one, a world-material id plus a slide flag.
  * Standing: characters stand on surfaces up to 65° (the game's ground limit) within a 0.5 m
    step; steeper surfaces are walls.
  * Not solid: grass, wires and other undrawn or cut-out parts have no collision surface, so
    they don't block.
  * Fallback: a level without collision data uses the drawn triangles instead.
* **Doors and gates.** Their leaves sit on sliding joints (joint type h_fd8f670c). Doors open
  when anyone is within 5 m, close again afterwards (the opening played backwards), and stop
  blocking while open. Gates with wall panels open from those instead (below).
  * Motion: the level's animations-<level>.xmb has an archetype-set named after each door's
    archetype, with one target per leaf (by part name) on a float channel: metres along the
    joint's axis per frame at 24 fps. Doors take 1.21 s (upper leaf 1.89 m, lower 1.70 m); the
    gate takes 4.96 s (3.92 m).
  * Sound: a sound-trigger stands beside each door (within 6 m). Signal 31 plays as it opens
    (h_1edc357e, metal_door_1a; the gate's h_e1b1f167, an 8.6 s grind) and signal 49 as it
    closes (h_f25b1597). These waves are streamed (resource-type 1) from the language wave bank
    `ml-sounds/en/<level>-en.tgz` at the offsets in the level's sound bank.
  * The gate's two leaves have a clip each in its set (h_10d3fde3 and h_0a48ec76, 4.96 s, one
    target each); each leaf takes its channel from whichever clip names its part.
* **Gates and their wall panels.** A gate with wall panels opens only from them:
  * The panels are `world-button-object`s (h_16f22d4b, type h_ee4c83c9, `reticule-action` 1)
    sending signal 14 (h_eaf8a35b).
  * A `router-trigger` lists the panels among its objects (h_0b6b92ba) and passes their signal
    on (h_e91cf6a8) to the gate's `anim-trigger`s. Those animate the gate object (their own
    h_0b6b92ba) with one clip per leaf on signal 14, once only (h_f136d22d 1, where the
    proximity doors' signal 50 / 42 entries say 2147483647), and send on to the gate's sound
    trigger. So a used gate opens and stays open. On sdm_e34: panels h_14643319 / h_1d63038f ->
    router h_1c61561d -> anim-triggers h_f993d7ef / h_e8ad85ec -> gate h_14690535.
    `Level::button_opens` follows that chain.
  * Using one (capture todo/gate button.mp4): from in front of its button (hardpoint
    h_10f81d34, facing the model's +z), within 2.5 m and looking within ~45° of it, hold E for
    0.5 s. Reach, angle and hold time are guesses; the capture shows no progress bar and no
    gate message. While it's usable, "Hold E to activate panel." shows under the health bar.
    That's the game's "Hold " + its X-button icon + "to activate %s." with "panel", and the
    key here in place of the icon. The blue target ring (the four-notch reticle texture
    h_1d2a68d0, tinted) sits on the button.
  * The button glows green: material h_f87dca06 is shader type h_1df292a1 (851 materials), a
    lit shader whose glow texture h_e01baa40 is tinted by its constant of the same name
    (0.276 0.914 0.220 here) and added as light.
  * `BF_DOOR_LOG=1` lists the panels, what each opens, and the gates with panels.
    `BF_TEST_USE=<s>` presses use at that time.
* **HUD text** (`play_text.rs`) is drawn in the game's own body font, atlas h_e4e4d2f4 (as the
  menu draws it), its glyphs 1 atlas pixel apart over a black outline and shadow, as in the
  captures. Colours sampled from them: messages and prompts pale blue (186 210 249), ammo and
  NEW orange (254 165 104), an item that can't be used red. Capitals are ~11.5 units (14 for the ammo).
* **Health pickups** (`play_pickups.rs`). Placed `inventory-object`s whose item type (objecttypes
  `<inventory>`, `Game::items`) has function-type 5 or 19. Function-type is the game's IFSET_
  enum, named in default.xbe's table at 0x3be3cc: 5 IFSET_GENERIC_HEALING, 14 IFSET_AMMO_BOX,
  16 IFSET_MINIGUN, 19 IFSET_POWERUP_MEDKIT, 20-23 the power-ups.
  * Medkit (h_f5123ace): taken by walking over it (within 1 m) into the squad's shared
    inventory, up to its stack-limit of 25, with its pickup-sound. When full, "<name> cannot
    pick up Medkit." shows and it stays.
  * Squad members take them too, running over one: a medkit into the same shared inventory (left
    alone when it's full, without the message), a Garo fruit for their own health (left alone
    at full health). `BF_TEST_TARGET=1` with a `BF_TEST_GOTO` start 8 m behind one stands the
    first squad member on it.
  * The item box (capture todo/medkits.mp4) shows one item at a time, a grenade type or the
    Medkit: its name at the bottom and the count top right, over its HUD icon (the grenade
    type's or Medkit's own h_e5ec3f1f: Frag fe20b919, Medkit f647bbef). A grenade's count sits
    at the right middle and is hidden while only one is carried (see **Grenades**). Both are pale
    blue while the item can be used and red while it can't (a medkit at full health; later an
    item the character can't use, such as Brutus and OrgSen). A
    newly taken kind is selected and marked NEW (orange) for 3 s. Tab steps to the next item
    carried; held, the item list opens around the box (todo/medic + intenvory use case.mp4):
    the other items leftward along the bottom (two), the other grenade types carried up the
    right edge (five), as the Frag and Sentry recordings' inventory overview stacks them,
    stepping round as the wheel picks. G uses the selected item. There's no separate medkit
    shortcut. An item that can't be used now shows a grey icon and red text.
  * Each group of medkits (within 3.5 m of one another, on the same level) has one soft green
    glow over its middle, all the time, marking where medkits are (todo/medkits glow.png). It
    faces the camera and is drawn a little toward it so the ground doesn't cut it off; a light
    lit the characters standing there green. The group's medkits are laid out in an even grid
    round its middle (rows of ceil(sqrt(n)), a short last row centred), each turned like the
    first give or take up to ~11 degrees (fixed per medkit), spaced by the medkit's measured
    footprint plus 30% so none overlap (the png's four in
    two rows; the levels place them up to ~3 m apart). No ring: the blue ring is for gate panels only. The
    effect-objects levels place beside pickups (type h_117c1805, ALE powerup_spawn) aren't
    shown.
  * Using a medkit (G) plays the stance's use_item overlay (Sc_w1_/Sc_w2_use_item, ~1 s; events
    1a6b4920 reach, 0a6e8f79 in hand, 19f8311b used). The used medkit (the carried type's model,
    h_192d5337's archetype) with its cross turned red (its texture's blue texels, and its
    cross glow red), is in the throwing hand from 0a6e8f79. At 19f8311b it heals and plays the
    medkit sound, and the medkit falls from the hand, bounces and stays on the ground (the
    last 12 do).
  * The medkit's cross glows: the model's second piece is a flat quad over the painted cross
    with the untextured glow material h_031724e6 (shader h_f539fe8c, glow 0.10 0.34 0.83,
    wrapper opacity 50). Flat untextured glows like it are drawn at full strength (the
    opacity is for the see-through glass shells round other pickups), as in the png.
  * Pickups are loose: every inventory-object without an idle effect, and the used medkits.
    * **At the start** each sits in the level's pose, tilted to the ground under it (up to
      ~25 degrees; steeper is a step, so it lies flat). It rests on its lowest point, not its
      origin: a model's origin can be its middle.
    * **Kicks and blasts.** A character walking into one kicks it out ahead, faster than
      they're going and off to the side it was on, with a hop, rolling the way it's sent
      (once per 0.6 s, so it isn't pushed along). A grenade blast within 1.5x its radius
      throws it.
    * **Shots.** Anyone's shot whose line passes through one (its bounding sphere, before
      the shot's end) knocks the first it meets: 2.5 m/s along the shot, a 1 m/s hop, and a
      turn from how far off its middle it was hit.
    * **Tumbling** (`rigid_step`). It moves as a rigid body:
      * its mass is spread like its bounding box;
      * it touches the ground with its model's outermost points, about 26: the furthest
        vertex toward each of a box's faces, edges and corners;
      * each point that goes into the ground gets an impulse there: a bounce (0.3, from
        landings over 1 m/s) and friction (up to 0.6 of the bounce).

      Because the push lands on a point, not the middle, it turns the object too: it tips
      over edges, flips off corners and rolls to a stop. It takes 8 sub-steps per 1/15 s.
    * **Coming to rest.** Still for 0.25 s on the ground, it's at rest, unless it's only
      balanced on a point or an edge: the touching points' narrower spread is under 2 cm,
      like the Garo fruit on its point. Then it's pushed over the way it leans. At rest it
      eases flush onto the face it stopped on over about 0.25 s, keeping its heading,
      instead of snapping upright.
    * Moving ones push off the others. The medkit glows stay where the group was placed.
    * The bounce, friction, drag and thresholds are the demo's choices, not the game's.
  * The medkit sound is h_1538baad, the Sound whose file is h_15331992 (named "92193315":
    those bytes as stored in sounds-<level>.xmb), the same in all 54 levels' banks, 1.5 s.
    30% of the time the character also says a line of their "healed" chatter (line_tag
    12c35d69, whose block's h_ea21ae4b is 30), with the sound. In todo/medic + intenvory use
    case.mp4 Tex says e707f108 at his use and Brutus nothing at his. `BF_TEST_HEALED=1` always
    says the line.
  * Each pickup adds a line below the middle of the screen, "2x Medkit", counted up while more
    of the same are taken and gone after 3 s. Status messages ("No need to heal") show just
    above them. They stay put on the screen (the capture has them by the character).
  * What a medkit heals: an item's h_0a811e94 is the health it restores and h_11884f2e the
    stamina (STAMINA POWER has 0 / 50 where HEALTH POWER has 50 / 0), not a respawn time. The
    placed Medkit (60) gives, by its pickup-archetype, the inventory Medkit h_192d5337 (80):
    using one heals 80. What the placed one's own 60 is for isn't known.
  * Healing Garo Fruit (h_e711067e, function 19): eaten when walked over, +40, left alone at
    full health. HEALTH / STAMINA POWER are also function 19, but they're power-ups with idle
    effects and aren't taken yet.
  * A taken pickup comes back after 30 s (a guess; the respawn time isn't found). Only the
    player picks up.
  * Test hooks: `BF_PICKUP_LOG=1` lists the pickups and each one taken or used.
    `BF_TEST_HEALTH=<hp>` starts hurt. `BF_TEST_MEDKIT=<s>` uses a medkit at that time. `BF_TEST_ITEM_LIST=1` holds the item list open.
* **Movement constants from default.xbe** (see `decompiled/xbe/ghidra/README.md`, "Player
  movement"):
  * Gravity is 18 m/s² and the jump takes off at 5.8 m/s. In the air, horizontal speed keeps
    0.99 of itself per game frame (taken as 30 Hz).
  * Walking off a drop of more than 0.5 m is a fall.
  * Landing after a drop of 5 m to 30 m costs 100 × ((drop − 5) / 25)² health (a 10.6 m fall
    cost 4.9).
* **Sliding** works as the game does it, and only on collision triangles flagged as slide
  surfaces (the canyon walls).
  * Building up: a slide amount (0–1) grows while you move. On ground steeper than 50° it grows
    at up to 8 per second, at the full rate from 55°; flagged ground always counts at least half
    rate. At 1 the slide starts.
  * Sliding: it carries on until the ground eases under 45°.
  * Speed: the slide velocity starts at (1 − normal y) × 8 downhill plus 5 m/s down, then
    builds by that push 3 times a second, up to 40 m/s. You move by it times the amount.
  * Ending: under an amount of 0.2 your own movement blends back in. A slide that's blocked (a
    wall, or flat flagged ground where it only presses down) ends.
  * What you see and hear: the slide clip (motion "slide", script Sc_w1_slide_idle), facing
    down the slope, the surface's slide sound, and the slide dust (effect type h_01513b0f, ALE
    effect slide_puff) at your feet.
  * Steep but not flagged: unflagged slopes up to 65° can be walked up.
* **Music.** The game streams `data/sounds/<level>.xwb` by level name (sdm_e34's is
  caspian_action20, 59.7 s, stereo Xbox ADPCM, 22050 Hz). A bank can hold a music track and an
  ambience bed; the level's sound bank gives each its Type (`h_f92fb2b9` music, `ambient`; for
  names it doesn't define, an `amb` prefix is ambience). The music loops at half volume with the
  bed under it at 80% of that; a bank without music plays all its beds. Before, the first wave
  played, which on sdm_e13 and sdm_m03 was the bed, so their music was missing:

  | Map | Music | Ambience |
  |---|---|---|
  | sdm_e01 | caspian_action1 | |
  | sdm_e10 | ferix_action8 | amb_ferix_02 |
  | sdm_e13 | tmp_full-on1a-f | amb_singe_02 |
  | sdm_e34 | caspian_action20 | |
  | sdm_e40 | creepy_estuary_mutant_01 | light_bird_bed_1 |
  | sdm_m03 | caspian_action5 | amb_casp_04 |
  | sdm_m07 | | amb_shad_int2, ambiencehellish-2ch |

  `BF_NO_MUSIC=1` turns it off, `BF_MUSIC_VOLUME=<0-1>` sets the level, and
  `BF_DUMP_MUSIC=<file.wav>` writes the track and exits.
* **Shots.** Each weapon's bullet definition names its own effects, and they're played from
  the effect library: `effect-name` is the shot in flight, h_08d28037 is where it lands.
  | Weapon | Speed | In flight | Hit |
  |---|---|---|---|
  | LZR-23 laser | 150 m/s | laser_bolt_s + laser_bolt_rifle_s, a glowing bolt | laserhit_s, laserhitb_* |
  | Laser pistol | 100 m/s | laser_bolt_s | laser hits |
  | Brutus's cutter | 120 m/s | cuttertrail_ribbon, a bluish ribbon trailing the shot, and cutter_flare | — |
  | Minigun, ballistic guns | instant | `tracer`, one particle shot from the muzzle at 113.6 m/s | sminigun_hit |
  | L-Shot | instant | tracer_snipe, a white beam drawn 200 m down the line in 0.38 s, smoke, rings | sminigun_hit |

  * Bolts carry their effect with them: the emitters are flagged "attached" (h_e2999ffd), so
    their particles move with the shot.
  * Hit effects play on world hits; hits on bodies bleed instead (play_fx).
  * A shot's damage (friendly fire) lands when it reaches the body: distance / speed later.
  * Beam appearances (class h_1f55f13e) are ribbons through their emitter's live particles in
    birth order, facing the camera. They have their own colour, alpha and width parameters, and
    their texture runs along v.
  * A frame's new particles are spread over the frame: along the emitter's animated offset
    (the sniper beam's sweep) and the effect's own movement (trails behind fast shots).
  * Effect libraries refer to their nodes by the case-kept name hash (e.g.
    "laserhit_Cone.emt"). Matching that recovered the emitters of 262 effect parts, the laser
    hits among them.
  * Test hooks:
    * `BF_TEST_FIRE=1` (or `2` for the second weapon) makes a `BF_TEST_GOTO` player aim and
      fire.
    * `BF_TEST_TARGET=1` stands the first squadmate still, 8 m in the line of fire.
    * `BF_CAPTURE_FPS=<n>` sets the capture's fixed step (default 15).
    * `bf_level` with `BF_FX_TEST=<effect name>` (and `BF_FX_MOVE=<m/s>`) runs one effect in
      front of the view.
* **Look.** The game lights each surface `texture × (ambient + key·N·L + fill·N·L)` with the
  level's own light colours, in gamma space, with no tone mapping. The demo does the same
  (`level_scene::spawn_lighting` and `console_look`):
  * Lights: the level's key (with shadows) and fill directional lights and its object ambient,
    given as sRGB colours.
  * Camera: tone mapping off, and an exposure where a light of 1.0 leaves a texture as it is.
  * Terrain: the level has a separate, dimmer key light for terrain (a bluish 0.44, 0.57, 0.65 on
    sdm_e34), so the terrain is tinted by its ratio to the objects' key light.

  Before this, an invented 9000 lux sun, a strong ambient and Bevy's default tone mapper (which
  desaturates) made everything pale. Point lights (muzzle flash, hit glows, explosions) are
  scaled to the new exposure by `POINT_LIGHT_SCALE`.
* **Power-ups.** Each power-up type's idle event (`<events><event state="1" h_ed0c9fac=…>`)
  names an effect type. Its objecttypes `<effect>` entry lists the ALE effects:
  | Power-up | Effect | What it draws |
  |---|---|---|
  | Brute (quad) | multi_quad_icon | godray.tga rays |
  | Force (shield) | multi_shield_icon | brokenring.tga ring, plus the animated texture "arcb" (4 × 4 frames at 30 fps) |
  | Health | multi_health_icon | red-flare.tga |
  | Stamina | multi_stamina_icon | blue-flare.tga |

  `src/ale_fx.rs` runs them from the effect library's own parameters (see that file).
  * These are "perp" particles: flat in the emitter's frame. The shield's arcs emitter is turned
    −87.6° about x to stand its arcs up.
  * Each particle keeps the appearance's turn at the moment it's born (about 220°/s), so the
    particles fan out round the icon.
* **Pickups.** They're stored at their spawn height in the level file; they now settle onto the
  floor below.
* **Compound joints.** A compound object's joints carry a rotation (a w-x-y-z quaternion),
  applied to the part (the wall sets turn pieces by 90° and 180°). A part sits at the joint's
  parent-point + child-point, both in the parent's frame; the rotation turns only the part about
  its own origin. Bulgar's compound `h_0baf091a` (hangar, bunk rooms: 19 pieces) is the one placed
  object with non-zero child-points, and only this reading puts every piece inside the building:
  read as parent-point − rotation × child-point, its roof bays landed outside the hangar and its
  turned bunks went through the walls (issue #12). `BF_OVERLAP_LOG=1` (bf_level) lists large
  objects whose boxes overlap much.
* **Winding.** Object triangles are wound to face along their stored normals, so a mirrored mesh
  isn't culled from the front.
* **The gate (h_0792d16a) and texture address modes.** Its two leaves (h_e783bd32 → mesh
  h_e01d9aa6, h_fcd25d86 → h_178bc697) are the same panel shifted 0.996 m, with matching UVs
  and no joint rotation. The mirroring is in the texture's sampling: each leaf runs its
  texture u 0..2 across, and the texture entry's `address-u` / `address-v` (h_1c82a17b /
  h_078bf0c1) is `TAM_MIRROR` (h_18712765), so the second half is the first mirrored and each
  leaf shows one symmetric brace. The other modes are `TAM_WRAP` (h_e3012d9e, most textures)
  and `TAM_CLAMP` (h_e041e0b9). Every level and model texture is now sampled by its own modes
  (`Game::texture_address`); 24 levels use mirroring somewhere.
* **Invisible or glowing materials.**
  * Power-ups (Brute / Force / Health / Stamina) use a placeholder cube whose material no level
    defines; it isn't drawn.
  * Untextured glow materials (type h_f539fe8c, e.g. the medkits' shell) are drawn as their
    glow colour at the wrapper material's opacity.
* **Hits.** Shots, grenades, decals, the DNA and the follow camera use the same collision.
