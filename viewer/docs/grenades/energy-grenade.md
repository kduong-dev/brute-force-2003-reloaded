*Part of [grenades/](README.md), the former page `grenades.md`.*

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
  the recording's squadmate went up and back at +0.6 s, was down ~1 s and got up). A body
  already down is thrown up and out: every body of its ragdoll brought up to 3 m/s along the
  throw (`Ragdoll::fling`, #259, [ragdoll.md](../ragdoll.md); the recording threw a corpse into the
  air). **A corpse** is the damage handler's request (#366's take06, read live), as a Frag's on
  one but with the Energy's type 6: factor 0 stays 0, so no all-body share, only the root
  takes its hit-point share (and the blast's own push of it): the root moves ~3 m/s along the
  blast and the corpse slides ~0.3-0.6 m and lies still (take06: 0.34 m, the demo's 0.3-0.6 m by the blast's distance) (`grenade::blast_on_corpse`;
  [ragdoll.md](../ragdoll.md)). The player's red tint comes with their strike.
* **A kill by it** (#366's take03 / take05: the damage record read at the CBT_DEAD handler) is
  an ordinary blast kill: damage type 6, push 2 (the bullet's h_e39b994b), the gate open, a bone
  body named, the way from the blast's centre to the victim, two requests; the body is
  thrown as a Frag's, mean 15.4 m/s (65 degrees up for a blast low beside him), the up-kick
  refused at 10 m/s (`strike`: `ragdoll::death_shoves` with the root body struck as the stand-in
  for the bone). The take timed the death ~0.2 s after the fuse ran out, which reads as the
  blast's own damage after its 0.05-0.25 s delay rather than the bolts' later strikes; the
  demo keeps #74's bolts (not changed: #366's shoves only).
* `BF_GRENADE_LOG` prints each bolt (to whom, how far, alive or dead), each strike (how long
  after the blast) and whether a corpse has a ragdoll for the blast's request to go to (a
  vaporised body or one not built yet has none); `BF_COMBAT_LOG` the damage and whether
  they were thrown down. `BF_RAGDOLL_LOG` prints the request and its push ("ragdoll shove",
  "ragdoll blast").

Verified (test map, 15 fps; scratchpad `en74/`):
`--test BF_TEST_GRENADE_TYPE=Energy BF_TEST_GOTO=0,10,0,10 BF_TEST_TARGET=1
BF_TEST_THROW=1.5,0.6,5.5,0.6 BF_TEST_TOGGLE_KILL=5.0 BF_CAMERA_PITCH=-0.45
BF_CAMERA_DISTANCE=8 BF_GRENADE_LOG=1 BF_COMBAT_LOG=1`: the first blast kills Brutus (instant
kill) and floors Hawk; instant kill off, the second (t 8.67) strikes Flint (2.3 m, 73 HP, thrown
up and back, down ~1 s, back up) and Hawk (2.7 m, killed) at +0.33 s and Brutus's corpse (6.9 m,
"body thrown" then; since #366 the corpse is shoved by the damage handler's request instead and
the log says whether it has a ragdoll) at +0.33 s. Side by side with the recording's fourth blast (frame 1772, every
4th frame to +0.87 s): the cyan wash at +0.07-0.2 s, the bolts from ~+0.27 s, strongest
+0.33-0.6 s, gone by ~0.8 s as recorded; strands end on the struck bodies. Differences: the
godray shafts show from ~+0.07 s to ~+0.33 s, the recording's from +0.27 s to +0.47 s; the
wash is whiter and the strands thinner and crisper than the recording's (its halo is wider);
some of the master effect's random bolts climb a few metres into the air (its emitter tumbles
them; the recording has a few such).
