*Part of [grenades/](README.md), the former page `grenades.md`.*

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
  olive); one it kills dies as from any hurt, **with no shove** (#366's take04, a Soldier at 1 m,
  1 HP: the lethal tick's damage record has the shove gate closed, flags 0xf3, because the
  cloud's flag 0x10, the Damage's h_0e373132, is what shuts it: FUN_00224510; no request and no
  up-kick, and the body falls where it stood from the first frame, [ragdoll.md](../ragdoll.md)).
  Squadmates don't leave or avoid it (not
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
