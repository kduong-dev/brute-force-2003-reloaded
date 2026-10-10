# Grenades (`src/bin/play_grenade.rs`)

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
    same (by name, see **Sonic grenade**).
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
  FUN_0022c4d0) but not implemented, and it doesn't explain the recordings. No damage from a
  blast without any (the Light); one whose damage is dealt over time (h_04ea9251 > 0: the Gas,
  whose recording shows none at once) leaves a poison cloud instead (see **The Gas's cloud**
  below). The Energy's damage comes with its bolts instead (see **Energy grenade**), the
  Sonic's with its ring (see **Sonic grenade**).
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
| Roller (#79) | set down (place_hi), rolls straight at 4.7 m/s with its rolling sound, bounces off walls, 25 s fuse, the Frag's effects + h_065168c9 | seeking |
| Sentry (#80) | set down (place_hi), lies there; goes off for a hostile within 3 m with no friend within 3 m, or when shot (see **Sentry**), or by a blast that takes its 1 hp (see **Interactive scenery**); exp-mine + light_explosion, h_145f09e5 | disarming an enemy's mine; the AI keeping clear of it; LEDs; the thrower's damage beyond the radius (below) |

Not done for any: ALE fields other than the Gas cloud's and the trail's rise (exp-lrg-air,
the gravity and turbulence fields: #101); decals don't
follow uneven ground (a plane along the slope under the middle: a big scorch on a bumpy
hillside is partly buried); the sounds' play-length; distance falloff for blast sounds beyond
a volume.

## Sentry (`src/bin/play_sentry.rs`)

The proximity mine (h_e5f1f063, function-type 8 IFSET_PROXIMITY_EXPLOSIVE): set down as the
Roller is, its `timer` 9999 s never runs out; what sets it off is the item set's handlers in
default.xbe (vtable 0x39bc50; read in the disassembly, Ghidra has no C for them):
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
  within 0.3 m across and 1.5 m up or down (0x147430). Which characters have the flag isn't
  read yet (only the test hook's `wise` hostile has it).
* It has 1 hitpoint: a shot (anyone's) whose line passes through its model's bounding sphere
  stops there and sets it off when it arrives. The shots are tested against the mines as they're
  made (play.rs's `update_player`, before friendly fire): the first thing on the line takes the
  shot - a teammate in front of the mine is hit and the mine isn't; a teammate behind it isn't
  hit. Bodies on the ground don't stop shots in the demo (they're shoved and the shot goes on),
  so a mine behind one is still struck, and bodies beyond the mine aren't shoved. The loose
  pickups (play_pickups.rs, after `update_player`) see the shortened shots.
* The demo has no enemies: the squad is one team, so the squad's mines never go off for the
  squad (as recorded: Tex walking or running over his own, standing beside it, Flint stepping
  onto it at 0.16 m). Only a test hook's hostile, the test map's enemy NPCs
  ([testtools.md](testtools.md)), or a shot, sets one off. Nobody can pick one back up.

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
  shot; Tex takes nothing (outside the radius, where the recording's Tex took ~15, above).

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
(todo/Sonic Grenade.mp4, the reference and tester agents' frames; the second blast at frame
449, camera still): a glowing dome growing from +0.03 s, flattening into a ground ring at
~0.25-0.3 s, godray shafts +0.27-0.85 s, the glow gone by ~1 s; hits at +0.17 s (~2 m),
+0.23 s and +0.47 s; no camera shake.

* **The damage comes with the ring** (an inference from those times): every living body within
  the 10 m when it goes off is hurt once, d / 11.68 m/s after it (sonic_grenade.emt's speed at
  its first key), no sooner than the other blasts' 0.1 s: +0.17 s at 2 m, +0.43 s at 5 m,
  +0.86 s at 10 m (on the first frame at or after that time). A body out of the radius by then
  is missed. Damage max falling to nothing at the radius (the other blasts' rule), times the
  damage-type factor.
* **The thrower** takes 0.653 x Damage max x (1 - d / 5 m): 28 HP at 2 m, nothing from 5 m.
  Fitted to the recording's two throws, not the game's formula: Tex lost ~28 HP at ~2 m (T1,
  below the data's minimum of 39) and nothing at ~5 m (T2: his health frame flashed red, the
  bar didn't move, no tint; with ~9.5 HP left a flat share would have killed him). A thrower at
  1, 3 and 4 m (a new shot) would settle the shape. The red tint comes only with a hit of
  1 HP or more on the player (`TINT_MIN`, a guess: the falloff still gives a few tenths of an
  HP at 5.0 m, measured from the pelvis, where T2 had no tint); it isn't scaled with the
  damage, since only the Frag's ~12.5 HP tint was measured.
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
  thrower stays standing, as Brutus did. Either is multiplied by the character's factor for
  damage-type 6 (**Damage types** above: Flint x2, the others 1); anyone else takes Damage max falling to
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
* `BF_TEST_DETONATE=<s>` sets off every grenade out at that time. (The Sentry's own hooks:
  see **Sentry**.)
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
* Sonic: see **Sonic grenade**.
* Light (`BF_TEST_GRENADE_TYPE=light`): see **Light grenade** above.
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
  identical (before #81, which changed the Sonic's).
* Energy: see **Energy grenade**.
