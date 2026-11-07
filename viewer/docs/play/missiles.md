*Part of [play/](README.md), the former page `play.md`.*

### Missiles: the Therm Sweeper (#73), the Sweeper V (#72) and the lobbed ones (#239)

`src/bin/play_missile.rs` (the camera shake in `src/bin/play_shake.rs`), the bullet's data in
`weapon::MeshBullet` (`src/bf/weapon.rs`), from default.xbe's projectile class
(0x226000-0x229000) and the weapon type's damage handler (0x22d0b0); every function cited here
is in [code-map.md](../code-map/README.md). The measurements are the tester's specs,
`todo/73-therm-sweeper/spec.md`, from takes 01-07 there, and `todo/72-sweeper-v/spec.md`; #73's
takes 08-23 (a real reload, self-damage at a wall, homing on two enemies and an off-line one, the
open air, a side view, cover) came after its spec and are compared under **Verified**.

* **Which guns.** bullet-type 2 is BTYPE_MESH_PROJECTILE (the enum at 0x3bd63c): FUN_0022f1d0
  launches the bullet's mesh-name object (FUN_0022fb50), and every one flies here
  (`missile::flies`). Those steered at a steady speed (h_0b03ece5, flag 4) are the Therm
  Sweeper, which splits (a `<mirv>`), and since #72 the Sweeper V, which doesn't; no other weapon
  in any level's data has the flag. Since #239 the iKhan-GPL's round flies here too, lobbed
  (below). The seven bullet-type 2 weapons with a projectile object in the data are these three
  and the hounds' four Breath Attacks (h_0edc1617, h_1492c676, h_11582931, h_1c6108cb: lobbed,
  `<mirv>` count 1 mode 0, 24-45 m/s, blasts of 3.5-8 m), which fly here since #83 (**The
  hounds' Breath Attacks**, below); the creatures' other natural weapons (weapon-type 7,
  WTYPE_NATURAL, the table at 0x3bf714: the Seers' Psionic Blast) are bullet-type 0, bolts
  (play_bolts.rs), not missiles.
* **Lobbed** (#239; `falls`, `GRAVITY`, `SPEED_DAMPING`): without h_0b03ece5, and with neither
  homing (flag 8) nor seeking (0x40), FUN_00226cc0 never steers the projectile (FUN_002266d0 is
  skipped, 0x226dd9-0x226dfc). Its body keeps the velocity FUN_00227ce0 gave it at the launch,
  the bullet's speed along the shot (vtable +0x58 adds the change through FUN_002b3b20: no mass
  term), and falls under the Ipion world's gravity, (0, -9.81, 0) (FUN_0016a250), its speed
  damped by 0.1 a second (the body template of FUN_00158cd0 / FUN_00158f80: 0.1 where Ipion's own
  template constructor, 0x2b4710, puts the speed damping's 0.01; #161's inference, by that
  layout), each game frame before its tick, as the physics step runs before the objects'
  (FUN_00169e20). **Which bodies fall** is the projectile object type's (`falls`): its `<base>`
  h_11e7d864 (base +0x35 -> the runtime type's +0xa8 bit 4, FUN_00154a20) has the root body's maker
  FUN_00158cd0 (game object vtable +0x158) take the body off the world's gravity controller
  (0x158ec0 -> FUN_0016a760: the core leaves the controller of priority 1000, vtable 0x38e7f4,
  0x2b4770, the one the Ipion environment makes beside its gravity at 0x2b49ff). It's set on
  exactly the two steady missiles' objects (h_e9327576, h_fa69b507) and on no other mesh
  projectile's (h_175a0adf, the breath objects h_0dc93533 and h_175340d6): the reviewer's lead,
  traced. The iKhan-GPL's (h_e9be70f0): 23 m/s, range 40, so a life of 1.74 s and a
  straight phase of 0.6 of it, 1.04 s; fired level from Brutus's muzzle, ~1.45 m up, it comes
  down 12 m out 0.53 s later. It goes off at its first contact (0x227b90: h_0bafbd6f set and
  n1 = n2 = 0; another projectile doesn't count), the world or a body, as any missile here: one
  blast. WM_MIRV_ON_APOGEE (its `<mirv>` mode 1) splits it instead on the tick its upward speed
  is under 0.5 m/s past its straight phase (0x2272b4-0x2272ea), when its life runs out, or at its
  proximity fuse (a hostile within its Damage radius 8 x 0.375 = 3 m that it has passed, looked
  for every 0.3 s), which sets its life to 0: 3 children h_175a0adf leave beside it, turned 14.4
  degrees (spread-max 0.16 x 90) at a random roll, **at the parent's speed then** (FUN_00227f80:
  without h_0b03ece5 a child's +0x208 is the length of the parent's velocity, 0x228748-0x228794;
  a steady parent's leave at the bullet's), and fall in their turn, NEUTRAL and without a target
  (no fuse), living 0.85-1.15 x 1.74 s, going off at their first contact or where they are when
  that runs out (h_0976a9dc). So a round fired at something short of ~12 m goes off on it as one
  blast; the bomblets come from the fuse when it passes an enemy, or from a shot more than ~10
  degrees up. Its node isn't turned (nothing steers it), so the drawn round keeps the turn it
  left with. The game runs a projectile's timers x5 while it's in a liquid (its +0x104, 0x226d22):
  not done.
* **An AI's lobbed rounds** (#83: `missile::aim_raise`, `missile::lob_holds`; the game's rules,
  read from the code, where #239's stand-in solved the flight for the target): fired straight at
  the target, a lobbed round falls short (e40's Shrike placement h_1f979a8a, which carries the
  iKhan-GPL, dropped its rounds ~5 m in front of itself and its own 8 m blasts killed it). The
  game's AI doesn't fire straight, unless it is a squad type's (below).
  * **The aim raise.** FUN_00017b60, a brain's set-up when its character is made, calls FUN_00017f80
    (0x17f80): when the gun in hand is a weapon (object-type 0x14) whose bullet-type (weapon data
    +0x40) is 2 and whose projectile object (the bullet's mesh-name, data +0xa8, looked up by
    FUN_00160da0) has no h_11e7d864 (its +0xa8 bit 4 clear: the round falls, `falls`) it sets the
    brain's bit 13 (0x3c4 |= 0x2000, 0x17fe6). The attack check FUN_00024f80 gives its gun the
    target's node as the aim point (FUN_0022dc60) plus FUN_00024c00's offset for a character
    target, and with the bit the offset is raised straight up (`test ch, 0x20` at 0x24e74) by
    0.5 x (1 + 13 x (min(d, 40) / 40)^2) m, d the distance between the two nodes: 0.5 m at the
    target's feet, 1.1 at 12 m, 2.1 at 20, 4.2 at 30, 7 from 40 m on ([0x40e740] = 40, [0x40e744]
    = 14, [0x3a4f78] = 1; the raise comes with the accuracy miss or without it). FUN_0022e5b0 turns
    the gun straight onto that point, led by the target's velocity (not in the demo) and nothing
    else adds a pitch. It is a cheap ballistic correction, right for a 30 m/s ball at 20-30 m
    (the fireball: it comes down within 4 m of its target there) and ever short for the slower
    ones. **Not for a squad brain:** FUN_00017b60 calls FUN_00017f80 only with the brain's bit 8
    clear, FUN_00016ff0 sets bit 8 at the construction for a type with h_e78efbb2
    (`CharacterType::player_type`: the squad, the controllable allies and every multiplayer
    type), FUN_0008ada0 (a squad member's creation) goes through the same FUN_00017b60, and
    FUN_00017f80 is the only writer of bit 13 (one `|= 0x2000` in the whole decompile). So the
    squad's Brutus with an iKhan-GPL, a marine, and **the deathmatch and squad-deathmatch hounds
    (h_e78efbb2 true on h_18ce0512, h_0c1886c0, h_15d1a23e, h_1f621983) get no raise**: they fire
    straight at the target (`missile::raises`); the campaign hounds (false) and e40's Shrike with
    the iKhan-GPL do. The raise's block also sits behind the character's skill hook (vtable
    +0x110: true skips the miss and the raise alike, 0x24c2b; the demo has no such skill). The
    distance is measured node to node before the accuracy miss (`Player::ai_hips`: the target's
    hips, its root node, since #244; it was a chest 1.1 m up before), as the refusal's is.
  * **The refusal.** FUN_00024f80 won't fire (reason 10) while the squared distance between the
    two nodes is at or under (1.5 x the gun's Damage radius, the weapon data's +0x24)^2, for
    every brain, a squad's too: 12 m for the gas ball, 10.5 for the fireball, 5.25 for the
    deathmatch balls, 12 for the iKhan-GPL
    (0x25281: [0x3a4fb4] = 1.5; with the brain's bit 9 the 0.01 m at [0x3a4f98] instead, not
    read). The demo holds the trigger by it for the lobbed guns (`lob_holds`); the game's check
    is every weapon's (the Sweeper V's 13.5 m, the Therm Sweeper's 6.75 m), which the demo's AI
    doesn't hold to.
* **The hounds' Breath Attacks** (#83; the leads are `todo/83-fire-breath/leads.md`; **no footage
  of a hound exists yet**, so everything here is the data and the code, checked in the demo only
  against itself): not a flame cone but a lobbed ball. The four weapons are weapon-type 7
  (WTYPE_NATURAL, the table at 0x3bf714) guns with a bullet-type 2 round, flown here like the
  iKhan-GPL's (above) since #83:

  | | Shrike Hound (e34, m09_a/c/x, m14_b) | Fire Hound (e13, m02_a/c, m04_a/c, m07_a/b/c) | dm / sdm Shrike Hound | dm / sdm Fire Hound |
  |---|---|---|---|---|
  | the weapon | h_1492c676 | h_0edc1617 | h_11582931 | h_1c6108cb |
  | its ball (object), trail type | h_175340d6, h_1bfe6f52 | h_0dc93533, h_146454be | h_175340d6 | h_0dc93533 |
  | speed, range, life | 24 m/s, 110 m, 4.58 s | 30, 90, 3 s | 45, 90, 2 s | 45, 90, 2 s |
  | rate (cooldown) | 0.6 / s (1.67 s) | 0.6 | 0.9 (1.11 s) | 1.75 (0.57 s) |
  | Damage, type | 28-31, 4 GAS | 19-21.75, 5 FLAME | 26, 4 | 13, 5 |
  | blast radius, share | 8 m, half the roll anywhere in | 7 m, half | 3.5 m, half | 3.5 m, the core rule (h_011cb154) |
  | blast effect type, scorch | h_099e42c1, h_096b8634 | h_01c43cbc, h_1e7e7d76 | same | same |
  | refuses within (1.5 x radius) | 12 m | 10.5 m | 5.25 m | 5.25 m |
  | aim raised (above) | yes | yes | no: a player type | no: a player type |

  Every one is `<mirv>` count 1 mode 0 (no split), proximity-fused (h_0bafbd6f) for a player's
  ball only, goes off where it strikes the world or a body or where it is when its life runs out
  (h_0976a9dc), and falls (its object has no h_11e7d864): the iKhan-GPL's flight, with the same
  blast, shake, scorch and damage code. A direct hit rolls Damage first (x the hit part, x the
  victim's damage-type factor, the difficulty step), then the blast adds half a roll to every
  body it reaches (no hit part). The damage types are the squad's own factors: GAS 1 and FLAME 1
  for Brutus, Tex and Hawk, Flint's GAS 0.05; the hounds take none of their own kind (GAS 0, FLAME
  0). **No burning:** the character damage path has no damage-type 5 branch and no burning state,
  so a fire hit is the roll x the factor and the blast's cloud, nothing left on the victim.
  * **The ball** is a projectile object (object-type 21) whose mesh h_e9e61b85 is a 0.5 m cube
    (bounding radius 0.433 m) with a material, h_0c6ac41d, that no materials file defines: the
    one geoset material of the objects of e34, e13, m02_a, m09_a (and the other levels read) with
    no definition. The demo draws nothing for such a mesh (`level_scene::visible`: the game draws
    nothing for the placeholder meshes that name a material no level defines, as for the
    power-ups' cube), so **the ball is its trail type's effects**, which `missile_kit` keeps:
    `gashound_fball` and `#1` (green smokecards, 11 a second of 0.4-0.9 m, over 0.4 s) or
    `firehound_fball` and `#1` (orange flame cards, 9 a second of 0.6-0.8 m), the type's light
    effect on it as it leaves (`spawn_missile`; none of the Therm's, the Sweeper V's or the
    iKhan-GPL's trail types has one): light_muzzle on the gas ball, the pale yellow flash of the
    Therm's muzzle (0.25 s, 0.5 growing to 4.6 m), light_hit_fhound on the fireball (yellow,
    0.2 s, 7.3 m); and its sound h_0021996c 0.2 s after the
    launch (the type's h_f724cb8c delay, now kept for a trail as for a blast's sounds). **An
    inference, not filmed**: if the game draws a 0.5 m box there, the shot list's A4 shows it;
    `BF_MISSILE_CUBES=1` draws it (in the demo's plain grey, lit) to compare.
    The air fields of these effects (`exp-lrg-air.fld` on the second trail and the sparks,
    `gashound_fball_exp_rise.fld` on the gas cloud: 2.2 m/s up) aren't run (`AIR_FIELDS`: the
    ones checked against a recording), so the cloud doesn't rise and the sparks fly their cones'
    6.5-12.5 m/s out.
  * **The blast**: the gas cloud is `gashound_fball_exp` (a translucent green card cloud, up to
    3.6 m a card, over ~1.1 s) and its sparks, with light_explosion; the fire's `firehound_fball_exp`
    (an added orange fireball to 3.3 m over 0.75 s, 0.2 s of sparks) with light_hit_fhound and
    its sound h_e297c1ef; both with the bullet's impact sound h_0997ea6b. The scorch is a 1 x 1 m
    black decal (25 s, fading over the last 5), drawn with the demo's half-size convention at
    2 x 2 m, laid DECAL_DELAY after the blast as every blast's.
  * **The fire rule.** The hound's attack is the generic one (GOAL_ATTACK -> FUN_00024f80, the
    hound's personality maps EVT_ENEMY_SEEN to it; what the hound goals 42 and 46 would do isn't
    run by anything): the refusal and the raised aim above (the raise for the campaign hounds
    only: the dm / sdm hounds are player types), the fire command, the gun's own cooldown
    (`period`: 1 / rate). The demo's AI fires its trigger in bursts and pauses of its
    own (play.rs `ai_fire`, the squad AI's), so a hound's rounds come every 1.7-2.5 s (1.9 on
    average) where the cooldown alone would give 1.67 s. **Not done, and the footage's** (shot
    list A3-A5): what the hound's two attack clips (`Sc_w1_gas_attack_1` 1.5 s with an event at
    0.633 s, `_2` 4.03 s with one at 0.767 s; FUN_00117660 turns both events into a shot) do to
    the timing: the demo fires at the fire loop's own tick and plays no attack clip; the hounds'
    spacing and their approach (#26); `gashound_bdy` (#375).
  * **Infinite ammunition.** ammo-count (the attribute's hash 0x1b545176, the weapon type's
    +0x28c, the data's +0xe4) -1 is an infinite clip: the weapon's clip counter (+0x1f8) starts
    at it, FUN_0022fe00 returns 1 for -1 without taking a round, and FUN_0022f2a0 only empties or
    reloads a clip it finds at 0, so it never does. The demo took it for a clip of 1
    (`ammo.max(1)`) and reloaded the hound after every round, ~1 s with the gun down (2.2 s
    between rounds in the first runs); its fire block (play.rs) now takes no round from a gun
    whose ammo-count is below 0. Every weapon-type 7 definition in the data has it: the hounds'
    four Breath Attacks and the Psionic Blasts (the Seers' bolts), whose NPCs lose the same
    phantom reload (a change outside the hounds: a Seer Follower NPC now fires through its
    bursts, 0.13-0.4 s apart, where it had a clip of one round to reload after each).
* **The iKhan-GPL** (#55; h_e9be70f0, WTYPE_RIFLE, Brutus's in the takes): 0.66 rounds a second
  from a clip of 16 (ammo-type 9, for which no level defines an ammo item: no reserve, no
  reload), its fire sound f3ccb53d, its muzzle effect h_0d18cac3 (grenade_launcher_smoke and
  grenade_launcher_muzzle, no light; compiled with the missiles' rules, as every mesh projectile
  gun's launch is), and the bracket crosshair h_12a2c478. Its round is the projectile object
  h_175a0adf (model h_13a8dfb8, 0.04 m), the yellow glow with a ribbon behind it: the attached
  effect h_f788f9c4 (mirvgrenade_trail: mirvgrenade_ribbon and mirvgrenade_flare) and its sound
  10318b29, lobbed (above). **Its bomblets** are its own `<mirv>` children, three more
  h_175a0adf: the fuse makes them as it passes an enemy within 3 m, the apogee when it's fired
  more than ~10 degrees up; each goes off on its own. Each blast (a direct hit's, a timeout's or a
  bomblet's): the explosion type h_e26237da (exp_mirvgrenade: a white dome, sparks and a flash;
  light_explosion; the sound 065168c9), the bullet's impact sound e404fac6 (in no level's sound
  bank: silent, as the Sweeper V's), Damage 18-20 EXPLOSION (as written, min 20 and max 18) in a
  radius of 8 m, half a roll to every body it reaches (h_011cb154 false), the scorch h_e1e26e25
  (1.5 x 1.5, 25 s) on the ground under it, and the view's shake (18 x 0.00769 = 0.14 within 56
  m). A direct hit takes a roll first. **The other "iKhan-GPL"**: h_1f4a5a0a carries the gun's
  stringtable name and description but is the bioreactive death's vaporise blast (WTYPE_DATA,
  Damage 1.1-1.5 x the victim's max HP, BIOREACTIVE, radius 7: the invisible grenade h_0569a8b4
  that FUN_001252b0 leaves at the body goes off with it, #147), never held, placed or picked up.
  `BF_TEST_WEAPON="iKhan-GPL"` took whichever of the two the weapons' map gave first (the blast
  about half the runs: Brutus then held its 7 mm placeholder); a label two definitions share now
  takes the one that isn't WTYPE_DATA (`shots::test_weapon_key`).
* **The Sweeper V** (#72; `h_ed3f6b0c`, the same machinery with other numbers, spec §2): one
  rocket a trigger cycle, 0.5 a second (60 game frames), clip 8 of the squad's AMMO_10 rockets.
  Its missile object h_fa69b507 is the Therm's model h_07d84a52 with its own attached effect
  h_fb438237 (the same mtrail, mtrail#1, mtrail-flare and hiss 0021996c); its explosion type
  h_e2a10c7d holds the Therm's six effects, light and sound; its muzzle effect (h_002779c8, the
  launch dust and light_muzzle) and scorch (h_e6868a8d) are the Therm's. It leaves at 60 m/s
  along the shot after the accuracy turn (accuracy 90 throughout: +-0.33 degrees) and flies
  **straight**: with h_07cdf040 and h_f69b7d4c false nothing steers it to a target or an aim
  point, so FUN_002266d0 steers it along its own velocity (flag 4) and holds it at 60 m/s, and
  with `<mirv>` count and mode 0 it never splits. It goes off where it strikes the world (not the
  level's invisible blockers and hinters, which Ipion's pair table keeps off every body but a
  character's: #298), a body
  or an object (a body struck takes a roll of 27-37 first), at its proximity fuse (Damage radius 9
  x 0.375 = 3.375 m), or where it is when its life runs out (range / speed = 200 / 60 = 3.33 s;
  h_f0a56297 0, so no second timer; h_0976a9dc). Its blast is 9 m: half a roll flat to every
  body it reaches (h_011cb154 false), x0.7 on the shooter's side; its centre is the bullet's
  radius, 0.3 m, off what it struck. The shake is 37 x 0.00769 = 0.285 within 31.5 m, to nothing
  at 63 m. Its impact sound h_f55abaf3 is in no level's bank: it plays nothing (the game's does
  the same: it asks and finds nothing). The Shrike Heavies that carry it (e40's placement
  h_17bc6519, 8 rockets) launch the same rockets at the squad, with no fuse (the fuse is a human
  player's missile's, FUN_00112800).
* **Launch**, on the fire loop's frame: the shot, the clip drop, the fire sound ed761bae and the
  muzzle effect in one game frame, as FUN_0022f2a0 does them (take21's side view: the clip digit
  and the missile at the muzzle on the same frame). The missile object h_e9327576 (model
  h_07d84a52; attached effect h_194dc78e: mtrail, mtrail#1, mtrail-flare and the hiss 0021996c)
  leaves the muzzle along the shot after the accuracy turn (+-1.23 degrees at accuracy 60) at
  the bullet's 30 m/s. Its target is the character on the crosshair's line before the world,
  if it's hostile (weapon +0x360, an inference: a squadmate in front of a hostile hides it, as
  it hides the crosshair's), its aim point the crosshair's world point (weapon +0x1ec). Life =
  range / speed = 20 / 30 = 0.667 s, the straight phase h_ea5327e6 x that = 0.1 s, then the
  second timer h_f0a56297, 5 s (FUN_00227dd0).
* **Flight** (FUN_00226cc0, in 30 Hz ticks: the move first, then the rest). Steering
  (FUN_002266d0): each tick removes half the velocity across the steering direction and pulls
  the speed 15% of the way to 30 m/s. The direction is along its velocity in the straight
  phase, then its target's root or its aim point. **The root** is where it homes on a character
  (`root_point`): FUN_00226cc0 refreshes the aim point each tick from the target's scene node
  (vtable +0x40, 0x10fda0; 0x2270fe), a character's model root node, which FUN_0023d4f0 puts at
  the feet + the root-height of the clip last started: the hips, ~1 m up (Brutus's rp_idle
  0.98, the Shrikes' idles 1.014 and 1.121), not the feet. An aim point within 1 m, or over 60
  degrees off its heading, is given up for one ahead along its velocity (velocity x 200: 6 km,
  0x227038), and the target with it; that test runs before the fuse, so a NEUTRAL child that
  gives up has no fuse and flies on to contact or its timeout. The trail's user parameter
  follows life / start (FUN_00228c10). No gravity: their objects' h_11e7d864 takes their bodies
  off it (**Lobbed** above), as the takes show (take20's children climbed straight at ~8.5 degrees
  for 168.6 m in 5.58 s). The steering wouldn't hold one up: the Sweeper V's, along its own
  velocity (flag 4 alone), takes nothing across it, and a missile under gravity would bend as
  freely as an unsteered round.
* **The trail's puffs** rise in place: the air fields linked to them, mtrail-rise (0.3 m/s up)
  and mtrail-rise#1 (1.0 m/s up, Approach 1.66 taken as 1), now run (`AIR_FIELDS`,
  `src/ale_fx.rs`). take21's side view shows a cloud round the missile from 0.1 s, then a line
  of puffs it leaves behind; without the fields their cones threw them 0.9 and 14.5 m/s back
  along the missile's way, past the holder.
* **The split** (h_11f515c5 = 2, WM_MIRV_ON_EXPLODE; FUN_00227f80), when its life runs out, 20 m
  out. The parent goes without a blast. Its 2 children (h_fe04300d) leave beside it, offset by
  the two missiles' bounding radii (0.32 m) along a side axis rolled at random (180 degrees on
  for the second), turned spread-max x 90 = 9 degrees toward it, at 30 m/s, living 0.85-1.15 x
  0.667 s and then 5 s more. The hostiles the split finds within 20 m and 30 degrees of the
  parent's velocity (its body's, vtable +0x54 = 0x159d70: the split builds its frame from it
  with FUN_0001d0c0) are dealt out to them in turn, the most central first; without any, each
  aims 200 m along the parent's velocity and comes back near parallel. **A child is NEUTRAL**:
  it keeps the projectile constructor's side 2 (FUN_002264e0 at 0x2265ec), since the split calls
  FUN_00227dd0, which copies the owner's team, before it copies the owner (0x2282bf, 0x22834a),
  and row 2 of the hostility table (0x3ffd80) is hostile only to PC_NO_TEAM (7), a side no
  character of the demo is on. So its own search (FUN_00226b80: once at the straight phase's
  end, target or not, 0x226d8e-0x226da7, and every tick while it has no target, 0x226e02; its
  timer 0.25 s; 60 degrees off its heading) finds nobody, which keeps the target it has: its
  only target is the one the split dealt it, and one that gives that up flies on along its
  velocity. The split searches with the parent's side, the shooter's (0x228199). The search
  (FUN_002237c0) measures its cone to each character's root and needs a clear ray to it,
  through a world query (FUN_000df0a0, flags 0xfffffffa) that takes the terrain and some
  dynamic objects but none of the level's placed geometry: the demo's ray meets only the
  terrain (`Arena::ray_terrain`); at the hips the ground they stand on isn't in the way.
* **Proximity fuse** (h_0bafbd6f): for a player's missile (FUN_00112800, read as "a human
  player's": medium confidence) with a target or a side other than NEUTRAL
  (0x22731c-0x22732d), a hostile (only its target, if it has one) within Damage radius x 0.375
  (1.69 m for the Therm, 3.375 m for the Sweeper V) of it, to their root, that it has passed.
  Its clock (+0x218) gains each tick's time while it's under 0.25 s and looks (and is cleared) on
  the tick it's 0.25 or more (the C of FUN_00226cc0's fuse block): every 9th tick at 30 Hz, 0.3
  s, which for the Sweeper V is every 18 m of flight, so a rocket passing a body is seen only if
  a check falls while it's within 3.375 m past it. It runs after the give-up and the timers.
  The parent then splits; a child (only while it holds the split's target) goes off; a Sweeper
  V's rocket goes off.
* **The heading stays the launch's** (a child's, the split's). The 60-degree test and the
  seek's cone read the node's -z (FUN_0001fdd0 at 0x226fdc), and each physics step
  (FUN_00169e20 -> 0x159eb0) sets the node back from the missile's body, whose turn is set only
  at the launch and the split (FUN_00227ce0's vtable +0x50): FUN_002266d0's turn of the node
  (0x2268b4-0x22690d) shows only in that frame's picture. So the drawn missile faces its
  velocity, and the tests read the direction it left in. Round 3's one-step lag is gone.
* **Hitting something** (the collision callback FUN_002278e0): the map's collision, a living body
  (not the shooter's) standing (play.rs's cylinder) or lying knocked down (its ragdoll's bones,
  each a ball 0.2 m round: `LYING_RADIUS`, the demo's shape; the game's missile strikes the
  ragdoll's bodies, where the plain shots pass over them), a breakable object, or a Sentry. A
  body struck takes a roll of Damage min..max (84-96, FUN_0022c4d0; min + U x (max - min) as
  written, FUN_0022c940: the iKhan-GPL's min 20 and max 18 roll 18-20), x the hit-location factor,
  x 0.7 on the shooter's side (FUN_0004b9c0), x the victim's difficulty step and damage-type
  factor. **The hit-location factor** (FUN_00125940 at 0x227a03, the bone from FUN_00115000;
  the missile's own +0x1cc is 1.0): the victim type's `damage-multiplier` (h_e7e9fcd3: 0.62 for
  the campaign squad, 1 for the Shrikes, and 1 for the sdm_bf squad of the squad deathmatch
  maps, which `Game::load_squad_library` reads with its skill data) x the struck hit part's
  bone's `Damage` from its motion set's `<collision>` list (the head x5, x3 for Brutus and the
  hounds; upper arms x0.8; forearms and calves x0.5; the hips, spine and thighs x1; no feet).
  The struck part is the first of the body's hit-part surfaces along the missile's way, from
  0.5 m before the contact (`STRUCK_BACK`; the ragdoll's nearest bone while knocked down): see
  **Damage** below, where the same factor, the step and the log are shared with every gun's
  shots. Blast shares take none of it (FUN_00224a90 passes 0). A breakable object struck takes
  a roll too (the scenery applies its damage-type factor), then the blast's share. A missile that hits something before its split goes off there
  without splitting (the takes' gate, the Shrike at 7 m). A child whose 5 s run out goes off where
  it is (h_0976a9dc, FUN_00227760).
* **The blast** (FUN_0022bfc0, dealt once by FUN_00224770) goes off at the missile's centre:
  the bullet's radius (0.1 m) off the surface it struck, along its normal. Its own roll,
  0.05-0.25 s later (at random), to every living body it reaches with nothing of the world
  between the blast (0.25 m off the surface) and the body's middle; a character behind cover
  isn't gathered at all (FUN_00224a90), so it takes nothing and isn't pushed (take22). With
  Damage h_011cb154 false (the Therm Sweeper's) that's half the roll at any distance
  (FUN_0022c4d0), x 0.7 on the shooter's side (the shooter too), x the damage-type factor. So a
  body struck directly takes a roll, then half another. **What it reaches** (#72): the game tests
  no distance. The blast is a physics ball of the Damage radius (FUN_0022c2e0, made as its delay
  runs out) and FUN_00224a90 walks the bodies the engine lists as overlapping it: a character
  is gathered when one of its collision-bone bodies is among them, its ray going from the centre
  to that bone, one entry per character (the analyst, todo/weapons-plan/answers-batch1.md §9).
  The demo meets the ball with the hit parts' collision surfaces as drawn
  (`damage::part_distances`, the same surfaces the shots strike), a part counting within the
  radius and 0.62 m more (`BLAST_PART_REACH`), and the line of sight goes to that part's bone;
  lying knocked down, the ragdoll's bones (`lying_parts`); a body without parts, its standing
  cylinder 1.35 m round (`BLAST_BODY_REACH`). The engine's own overlap rule isn't in the game's
  code, so both margins are fits to three takes: Tex was hurt by the Sweeper V's blast at the gate
  10.2 m away six times in six for a 9 m radius (#72 take01), and by the Therm's at 5 m but not at
  6 m for 4.5 m (#73 takes 15, 13). With Tex aiming at the blast, his arms out, the demo puts his
  nearest part 9.50-9.57, ~4.2 and 5.18-5.20 m from those blasts (his feet axis 10.30, 4.94, 5.94):
  a margin in [0.57, 0.68) for the parts, [1.30, 1.44) for the cylinder, of which these are the
  middles. #73's 0.6 m cylinder, fitted to the Therm's takes alone, fell short at the Sweeper V's
  10 m. The analyst's live test (a frozen enemy facing the blast against side-on, at r + 0.8 and
  r + 1.2 m) would tell whether the arms decide. Loose pickups are thrown. The scenery takes the same
  share, flat (`ObjectBlast::flat`): the blast's entry for an object other than a character is
  its own centre (FUN_00224a90), so FUN_0022c4d0 sees it at distance 0: half the roll here
  (h_011cb154 false), the whole roll when true, no falloff. (The grenades' scenery keeps its
  falloff: their path isn't checked against this one.)
* **No knockdown** (`hurt_still`): in the takes no hit or blast of a missile knocks its victim
  down or shoves it (Tex hurt by his own blasts, takes 10-16; frozen enemies hit directly and by
  blasts, takes 06, 18, 19, 23), where play.rs's `hurt` floors a body for 15 HP or more two
  times in three. The game's knockdown rule isn't traced. A kill still throws the body (the
  death push). An enemy of a type of its own reacts by its personality whatever hurt it
  ([enemies.md](../enemies.md), "Hit reactions").
* **The explosion** (FUN_0022cfc0): the bullet's effect type h_173132f0 (exp-rocket-dirt, -fire,
  -fire#1, -shrap, -flash, -fire-add, light_explosion and the sound 0ab060f5) at the blast
  point, turned with its -z along the missile's way (FUN_0022cbe0's FUN_0001d0c0 frame: y up for
  a level shot); a timeout's upright (FUN_0022cea0). It's compiled with the grenades' recorded
  rules (exp-rocket-* are exp-lrg-*'s siblings) and two of the game's own, for the Therm
  Sweeper's trail and explosion only (`AleAssets::load_recorded_facing`):
  * **Facing 3** (`FACING`, `src/ale_fx.rs`; also the muzzle effects', above):
    exp-rocket-add.app and exp-rocket-shrap.rect.app are AngleFadeOut ("perp") appearances with
    GeneralApp_FacingType (e3542300) 3: a quad stretched along its particle's motion (+z of its
    frame for one born still) and turned about it to face the camera, as the particle vertex
    shader (0x3b8bf8) builds it; facings 0, 1 and 2 are the particle's transform, the camera
    and the plane square to its motion. The takes' fireball is upright rectangles facing the
    camera, where lying across y they drew long horizontal streaks edge-on. The flag the demo
    called PERP (0d645074) is GeneralApp_AngleFadeOut: the shader fades a quad seen within 20
    degrees of edge-on, drawn since #286 for every effect ([maps.md](../maps/ale-blends-and-particle-facing.md), **Particle
    facing**): here exp-rocket-add's quads seen along their motion. Checked at take04's spot
    (#286, the recipe below, beside main's build): the fireball as before, a little less whited
    out where its quads come at the eye (scratchpad `t286dev/side_therm.png`).
  * **Shapes** (`SHAPE`, `src/ale_fx.rs`): GeneralApp_ShapeType (17d45550) picks the particle's
    vertex list (the table at 0x402a4c): 2, a quad from -1 to +1 (twice its Scale across), 3, a
    quad standing on the particle, 5, a quad from -0.5 to 0.5. The demo drew every quad as a 5;
    these effects draw 2 twice as big and 3 standing on its particle. The other effects keep
    the half-size quads (411 of the library's appearances are shape 2): their own check.
  * The Frag's exp-lrg-add.app and exp-lrg-shrap.rect.app are facing 3 too, drawn so since #286
    (grenades.md, "The blast").
  * light_explosion lights the scene as before. The effect types with a light-effect also list
    a light object type in h_f8c6a8b4 (h_09020c30, a point light of range 10, for the Therm
    Sweeper's muzzle and explosion); that the light effect drives it is an inference from the
    data.
  * **The count and the distance LOD** (`Compiled::counted`, `game_count`, `LOD_TAN` in
    `src/ale_fx.rs`), for the Therm Sweeper's trail, explosion and launch dust:
    * Each effect object has a distance LOD from the camera (the render cull FUN_000e7850 ->
      FUN_00156050): d = tan(35 degrees: the game camera's 70-degree view) x the distance past
      the effect type's bounding sphere (FUN_00219ed0: centre h_e99750a9 in the effect's
      frame, radius h_1ab09530; the explosion's (-1.99, -3.21, -1.67), 25 m; the trail's
      radius 0; the dust's (0.08, 0.19, 4.13), 5.21 m; `EffectType::bounds`); LOD 1 to 10 m,
      falling to 0 at 60 m (0x2194d0, 0x2194e0; x (2 - w) / 2 for the world's load and x 1 for
      one view, both 1 in the demo).
    * Each emitter's new particles (FUN_00065800), once a game frame (30 a second,
      `GAME_FRAME`) whatever the frame rate: the whole number of its frequency's over the time
      since its last batch (the frequency read at its end), its initial particles on its first
      update, plus each emit count key passed (cut to a whole number each time), then x its
      Emitter_LODCurve at that LOD (fe077e05, 0x659ed-0x65a12), cut again; the fractions are
      thrown away. Cut each update, the count depends on the step: round 3 stepped it once a
      render frame, and exp-rocket-fire made 1 sprite at 15 and 60 fps where 30 gives 2. A game
      frame's batch is born at its end, spread over its part of the effect's move, and starts
      back by its age along its way (`animate` moves every particle by the whole render frame).
      So exp-rocket-flash makes 3 particles (2 at 0.076 s, 1 at 0.16 s) up to ~72 m and none
      past ~94 m; the launch dust's 18 a second for 0.43 s make one card, at ~0.13 s. The
      initial particles come once (the flag the emitter's constructor FUN_00065640 sets). A
      trail emitter starting its 4 s over (a child's flight past 4 s) counts from 0 again, the
      demo's choice (the game's loop of an emitter's time isn't traced; it may stop it at its
      lifespan): its rate starts over at 0 for 0.07-0.09 s, so the trail has a ~0.2 s gap there.
    * A missile's trail is placed from the missile's own transform each frame, so each game
      frame's particles are born along the missile's move up to where it is then; read from the
      last frame's placement, they were born a frame behind it (2 m at 15 fps). The game's:
      its world update runs the physics step, which moves the missile (FUN_00169e20 at
      0xd9ad4), before the objects' ticks (FUN_000d9840 at 0xd9d0b), where the trail's
      (0x21b7a0) follows the missile and births along its move since the last frame. Like
      every effect on a parent just made, it starts on the frame after the missile's, when the
      missile starts to move, so its time keeps with the flight.
    * A particle's size is its Scale x its Appearance_LODCurve at that LOD (0c53b008; 1 on these
      effects) x the shape.
    * Every other effect keeps the demo's count (a rate's fraction carried over, an emit
      count's burst at the start) and no LOD: the general rule is a ticket of its own.
* **The scorch** (bullet h_06a27365 = h_e6868a8d: 5 x 5, RGBA 10 10 10 200, 25 s, fading over the
  last 5), from the code: the damage handler (0x22d0b0) lays a hit's decal (FUN_0022ca20) on a
  direct hit only when the Damage radius is 0.05 m or less (bullet holes). A blast weapon's
  comes from its blast's pass over what it reaches (FUN_00224770 back into the handler), whose
  entry for the static world sits at the blast's centre with its normal straight up
  (FUN_00224a90): the decal is laid round the blast point and projected down, on the ground
  under it, after a wall hit, a body hit (take03: under the enemy struck, from f256), a timeout
  or the fuse alike. The demo lays its flat decal on the ground below the blast point if that's
  within the radius (the projection's reach isn't traced), its size read as half sizes, a 10 m
  quad as the grenades' (the takes' pair on the cliff, ~10 x 5 m). The pass comes with the
  blast's damage, 0.05-0.25 s after the explosion; the demo lays it 0.7 s after (`DECAL_DELAY`,
  the grenades'): laid that early, the dark decal shows through the demo's paler added fireball
  as a hard dark disc (#104). The takes show it from ~0.85-1.0 s, once their fireball clears.
  At take04's cliff foot it showed only as a straight-edged wedge: the terrain's layers were drawn
  over it (see the terrain layers below); now it's a dark blot there, with the flat quad's
  straight edge where it leaves the curved rock.
* **The camera shake** (`play_shake.rs`): every missile blast shakes the player's view
  (FUN_00228c70, whose only callers are the missile's two explosion paths, 0x227867 and
  0x227b1d: a grenade's blast doesn't shake the view through it). A camera within 7 x the Damage
  radius (31.5 m) gets clamp(Damage max x 0.00769, 0, 1) = 0.74 x f, f being 1 within half that
  reach and falling to 0 at its edge (nothing when f is 0.1 or less there), held 0.5 s, then
  falling to nothing over 0.75 f s (FUN_0006bdd0). Each game frame the camera's update (0x6c780:
  FUN_00073d20, FUN_00073c00, FUN_00073a80) turns the view by a random pitch and yaw within
  +-0.025 x the amplitude rad and a roll within half that (FUN_00072010, FUN_000720d0,
  FUN_00073d80), new each frame. The view only, the demo's choice: the aim keeps the unshaken
  camera. The game multiplies the turn into the camera's own frame (0x6c54a, FUN_00073b20, the
  camera's vtable +0x3c); whether its crosshair ray reads that frame isn't traced. A weaker kick
  doesn't replace a stronger one still running. An effect type's own shake list (the Supermutant's
  pound, the e34 and m09 explosions: FUN_0021b620) holds the camera's one-frame amplitude on every
  game frame it runs, `CameraShake::hold_each_frame` (#373, [effects.md](../effects.md)).
* **The terrain's layers sort first** (`TERRAIN_LAYER_SORT`, `src/level_scene.rs`; see
  [levels.md](../levels/terrain.md)). Bevy sorts transparent meshes by their origin's depth, and a terrain
  layer's origin is the world's: where that origin was nearer the camera than an effect, the
  layer was drawn after it and painted the ground over it. At take04's cliff the blasts and the
  launch glow vanished that way (the tester's "no blast pixels over 200"), and the scorch showed
  only where the rock layer's mask let it through.
* **The difficulty step** (#127, see **Damage** below): every hit in the takes' HP logs is x1.4
  the code's rule on enemies and x0.6 on the squad, the character damage handler's (0x125600)
  1 - 0.1n and 1 + 0.1n on STANDARD (n = -4). The direct hits and the blasts here take it: on
  STANDARD, the demo's default on the missions and the test map (the multiplayer maps have
  none), Tex's own blast is 0.5 x 0.7 x 0.6 = 0.21 of a roll (takes
  10-16: 18.2-20.1), take03's direct x1.4 of a roll (118-134). The numbers under **Verified**
  below are #73's, from before the step (x1 there: 32.8 for Tex's own blast, 93.6 and 42.7 for
  take03's geometry) and before the hit-part surfaces (the bones the homing takes' children
  struck were then the nearest to the contact); **Damage** has the checks since.
* **Not done**: the AI's 75 m noise (FUN_0004d490), the damage event (FUN_001fe3d0), the second
  sound list h_03dc7b9a, the red enemy crosshair (a shared HUD feature).
