# The ragdoll (`src/bin/play_ragdoll.rs`, #259)

Every character's body at death, the squad's and the enemies' alike, and the squad's
knock-down, is the game's ragdoll: its bodies, joints, masses, damping, physics step, sleep and
pushes as #280's study and #259 read them from default.xbe (`todo/280-ragdoll/answer.md`,
`spec.md`; the rows in [code-map.md](code-map.md)), simulated by a small rigid-body solver of the
demo's own. It replaced the verlet point masses (#94) and #129's enemy branch of them. play.md's
death paragraph, [enemies.md](enemies.md) "Death", squad.md's knock-down and skills.md's Vengar
charge point here.

## What the game does

* **The build** (FUN_00127290, one a frame from the world's queue: FUN_000dfff0, FUN_000d9a20):
  the guns drop (FUN_0014dfd0); the whole skeleton is disconnected from the animation (node flag
  0x20, FUN_00127160); each hit part's body (FUN_00114010, FUN_00114d00: an Ipion body from the
  part's collision surface, one for each of the type's `<collision>` bones, eleven on a biped) is
  beamed onto its bone (FUN_00169140) and its motion controller deleted; each ancestor of a hit
  part gets a 0.05 m ball (FUN_00114b80 -> FUN_002cce90), the character's node too (the root
  body, char+0x500). A bone below the lowest hit part of its chain gets nothing.
* **Masses and inertia**: 9 kg for the root and for any body whose bone hangs from the
  character's node (Bip01), 4.5 kg for the rest (0x40cbe8 and half of it): 90 kg a biped. The
  inertia is set to 0.5 (the root) or 0.25 times that mass: isotropic, 4.5 and 1.125 kg m^2
  (FUN_00127290 writes the core's inertia, FUN_002bb410 its inverse). A 4.5 kg forearm turns like
  a 1.7 m rod: every limb is slow to swing.
* **Damping**: linear 0.95 and angular 0.5 a second (0x40cbe0, 0x40cbdc into the cores), taken
  each step as `v *= 1 - k dt` (FUN_002baa70), 0.1 more once a body is calm (FUN_002ba9b0).
* **The world**: Ipion at 66 steps a second (FUN_002b4780), gravity 9.81 (FUN_0016a250); a
  ragdoll body's material against the ground gives friction 2 x 2 = 4 (2 x 3 = 6 on some
  grounds) and restitution 0.2 x 0.2 = 0.04 (the world's material table, 0x2bc0b0, 0x2bc0e0).
* **Joints** (FUN_00113120: a constraint per body with its parent's, by the child bone's name):
  the hip (Thigh) x -45..45, y -60..110, z -20..20 degrees; the knee (Calf) y -90..0, a one-way
  hinge; the HorseLink, the ankle, Spine1-3, Neck1, the head (-45..45 on each), the shoulder
  (UpperArm) x -55..55, y -40..100, z -45..80; the elbow (Forearm) y -100..0; the wrist; eight
  more pairs of other skeletons. A bone not in its table (Bip01, the pelvis, the spine's base,
  the neck, the clavicles) gets type 1 on its three rotation axes, and type 1 is a fixed axis:
  the constraint template's constructor (FUN_002b8f20) sets the three translations to 1 and the
  three rotations to 0, a ball socket (so 0 is free and 2 limited, with the ranges). The hips
  (the node, Bip01, the pelvis and the spine's base) and the chest (Spine2, the neck and the
  clavicles) are rigid pieces. **Correction:** #280's study read type 1 as free.
* **The start**: an impulse of 9 x the character's velocity on every body at its centre
  (FUN_002b3bc0), only when that velocity isn't 0. A running character's 9 kg bodies leave at its
  speed and the 4.5 kg ones at twice it: the whole body at 1.8 times it. A body that stood still
  gets no push, and Ipion makes its objects out of the simulation (FUN_002b2f40: movement state
  8), so it stays as it stood until a push or a moving object's touch revives it (FUN_002b3c60,
  FUN_002bb1e0).
* **Sleep**: every 15-19 steps (a simulation unit's step, FUN_002be4c0, counts it down) the
  movement check looks at each body (FUN_002bb980): calm when it has gone at most 1 cm and its
  far end at most ~7 mm since its short reference, still once calm for more than 0.3 s (env+0x90);
  otherwise its short reference moves to where it is, and a body that in 4 s hasn't gone more than
  10 cm, nor its far end ~28 cm, from its long reference counts as still too. When every body of
  the ragdoll is still, it's frozen, its speeds zeroed (FUN_002be350 -> FUN_002bb0e0), until
  something pushes it.
* **Pushes** (FUN_000e00b0 queues, FUN_00127c60 applies): every body but the one struck gets
  min(0.5 x magnitude x factor, 75) N s at its centre, the one struck min(8 x magnitude x factor,
  300) more at the point (a factor of at least 3 for an explosion; times the world's load, 1 in a
  quiet scene). The Vengar's charge queues magnitude 75 and factor 2 along his velocity and 2 up:
  75 N s on every body, ~15 m/s for the whole body. The CBT_DEAD handler queues two on every
  death (FUN_00115f90: the hit's, magnitude 2 x the damage with the weapon's push as the factor;
  an up-kick, magnitude 1 and factor 36, 18 N s a body, ~3.6 m/s up), which no take shows. A
  blast pushes a corpse's root body 1.5 x (1.2 - d / r)^2 x the damage N s away from it when the
  weapon pushes (FUN_00224a90).
* **Where the corpse is**: its root body (0x1271a0 writes it into the character's node after
  each step): the death camera, the blood pool and the memory chip go by it.

## The demo's model

* **Bodies and pieces.** The game's bodies are made as the game makes them (`Ragdoll::new`), each
  with its mass, inertia and centre (the part's mass centre, `Character::part_mass_centre`), then
  the two sides of each rigid joint are put together into one solver body (`Body`): their masses
  summed, their inertias with them (each its own and its offset's, isotropic in the mean,
  2/3 m r^2). A biped's 18 bodies come out as 12 pieces with 11 limited joints, a Shrike's too;
  a 25-bone creature with a Spine3 as 19 bodies in 13 pieces.
* **Joints.** A joint holds its two bodies together at the child bone's origin and keeps the
  child bone's turn within FUN_00113120's ranges: its twist about the child bone's x and its
  swing's parts about y and z, measured from the pose at death (an Ipion constraint's frames are
  made from the bodies as they are, FUN_002b9370), in the child bone's own frame. Which way the
  game's axes point isn't traced (the frame comes from the parent node, FUN_00039510): this
  reading bends knees and elbows the way they were bent and starts every joint inside its range.
* **Bones without a body** (the feet, toes, hands and fingers below the lowest hit part, a
  Shrike's HorseLink, and every bone of the face, the hair and the props) keep their pose at
  death under their parent, and a bone with a body takes its body's turn at its own place under
  its parent: no bone stretches or slides against its neighbours, so faces and limbs don't morph
  as the verlet body's did.
* **Contacts.** Each body's points (its surfaces' corners, at most 26; a ball by its centre and
  radius) against the level's faces round it (`Arena::faces_near`) that the body is in front of,
  over the face (2 cm past its edges counts); on the flat test floor without scenery, a plane
  where the feet were. A point already in the ground at the start comes out at most 0.5 m/s (a
  Shrike's calf surface starts 3 cm under the floor). Friction 4 and restitution 0.04
  everywhere. Bodies of one ragdoll don't meet each other, nor the living, the loose objects or
  the dropped guns (spec S2.5).
* **The hold.** A body that stood still (under 5 cm/s) holds its pose 0.35 s (`REST_WAKE`, a fit
  to takes 17-22: 0.35-0.45 s) unless something pushes it first: the demo's dropped guns don't
  touch the ragdoll, so this stands in for what wakes the game's (the gun reaching the feet, a
  guess).
* **The movement check** runs every 17 steps (the game's 15-19 at random) with the game's
  thresholds (`movement_check`), and its calm state adds the game's 0.1 to both dampings.
* **The knock-down** (the squad's own: the game's is a clip, #129) starts the same body with the
  demo's old throw, the hit's push x 1.6 and 2.5 m/s down, both by height (`Start::Knock`); the
  character gets up from where its root lies after 0.7 s.
* **Shots and the Energy grenade.** A shot through a body on the ground (dead or knocked down)
  brings the part it meets first up to 3 m/s along the shot at that point, no faster, and it
  bleeds there (spec S6.5: the demo's, no such code was found; #129 take14's lying Heavy shifts
  as it's killed). The Energy grenade's bolt throws a body on the ground up and out at 3 m/s,
  every body brought up to it (`Ragdoll::fling`, the demo's).

## The solver

Extended position-based dynamics (Müller et al., "Detailed Rigid Body Simulation with Extended
Position Based Dynamics", 2020), 16 substeps a 1/66 s step. Each substep moves the bodies by their
velocities and gravity, pulls each joint's two points together and its child back into its
ranges, pushes each contact point out of its face, then takes the velocities from the moves:
a point that struck its face leaves it at 0.04 of the speed it came in at (none under 2 g dt),
and its sliding is taken away up to 4 x its push. Damping and the movement check come once a
step, as Ipion's. The game's isotropic inertia makes every turn's share a plain one. 16 substeps
hold a 90 kg body's joints to under a millimetre (`BF_RAGDOLL_LOG`'s joint gap), and a pendulum
of the game's limb swings at its 2.42 s period (a unit test): the solver adds no stiffness or
drag of its own.

A crate was weighed: rapier or avian would want the level's triangles mirrored into their own
world and kept in step with the doors and breakables `Arena` already tracks, for a dozen bodies a
character, and neither is Ipion's solver (a linear system over a ragdoll's joints each step,
FUN_002db0a0 / FUN_002db5a0) either. The demo's solver is one module with no new dependency,
takes the game's inertias, damping and sleep rule as they are, and reads the arena directly.

## Test hooks

* `BF_RAGDOLL_LOG=1`: at the build, the bodies (their parts, mass, inertia, contact points,
  triangles and balls, how high the lowest point is), the joints (their ranges, which way a bend
  takes the limb's end) and the start; then a line a frame for 3 s and every half second after
  (the root's place and speed, the fastest body and spin, the trunk's lowest point over the
  floor, the worst joint gap, the contacts, how many bodies are calm and still, asleep); each
  shove and blast, and each shot into a body (play.rs).
* `BF_RAGDOLL_DRAW=1`: every ragdoll's bodies (green surfaces, yellow balls, the root's piece red)
  and joints (blue).
* Deaths and knocks to watch: `BF_TEST_DIE=<s>` ([play.md](play.md)), `BF_TEST_KNOCK=<s>`, a
  frozen enemy shot (`BF_TEST_NPC=...,frozen,...` with `BF_TEST_TRIGGER`, [enemies.md](enemies.md)),
  `BF_TEST_FREECAM` to look ([testtools.md](testtools.md); it holds the player still while it's
  on, so for a running death start it after the death).

## Verified

Captures at 15 fps with `BF_MUTE=1 BF_BACKGROUND=1` and `BF_RAGDOLL_LOG=1`, against the takes
(`todo/280-ragdoll/answer.md` §3). Times are after the hit. "Moves": a body first faster than
0.1 m/s; "down": the trunk's lowest point (the root's piece and the chest's) first within 5 cm of
the floor; "quiet": the last frame with a body faster than 0.3 m/s or turning faster than
1 rad/s.

| case | moves | down | quiet / frozen | root moved | takes (moves, down, still) |
|---|---|---|---|---|---|
| Shrike Soldier, frozen (take22's spot: `--test BF_MAP=e34 BF_CHARACTER=1 BF_TEST_GOTO=-53.6,30.8,-53.71,31.49 BF_CAMERA_PITCH=-0.12 BF_TEST_NPC=0.3,h_e9a41520,enemy,frozen,-54.6,37.8 BF_TEST_TRIGGER=1.5-1.7`) | 0.40 | 0.93 | 3.07 / not by 7.5 s (8 of 12 still at 5.5) | 0.49 m | take22: 0.40, 0.85, ~1.95; takes 18, 20: 0.40-0.45, 0.85, 1.55-1.7 |
| Shrike Heavy, frozen (enemies.md's "Death" command) | 0.40 | 1.40 | 4.53 / 8.07 | 0.57 m | takes 17, 19, 21: 0.35-0.45, 0.55-0.83, 1.5-1.8 |
| Tex, an NPC, frozen (`--test BF_MAP=flat BF_CHARACTER=1 BF_TEST_GOTO=0,8,0,7.3 BF_TEST_NPC=0.3,tex,enemy,frozen,0.15,3 BF_TEST_TRIGGER=1.5-1.7`) | 0.40 | 1.47 | 4.07 / not by 8.5 s | 1.14 m | take05 (Tex standing): 0.5, 0.85 |
| Tex, the player, running at 4.3 m/s (`--test BF_MAP=flat BF_TEST_GOTO=0,20,0,-20 BF_TEST_DIE=2.2 BF_TEST_FREECAM=2.25,5.5,0.6,11,90,-10`) | at once | 0.93 | ~1.5 | 3.4 m on | FF2 18 s: 0.2, 0.5, 0.8 |

* **No morphing** (main's build against this one, close-ups of Tex and the Shrike Soldier
  through the fall and at rest): the verlet body's face and limbs smeared and stretched as its
  points were pulled about; now every bone keeps its shape and the face stays as it was at death.
* **Bodies land and stay**: within 0.5-1.1 m of where they stood (the takes: within ~1 m), no
  bounce and no slide once down.
* **The Vengar's charge** (skills.md's test-map command with 125 HP): "killed: thrown", the shove
  of 75 N s on every body along his way and up; the body leaves at 13 m/s and goes 11 m across
  the open floor. Against e34's gate panel 3.8 m behind (`--test BF_MAP=e34 BF_CHARACTER=0
  BF_TEST_NPC=0.5,tex,enemy,frozen,-54.5,37.5 BF_TEST_NPC_HP=1.0,1,125 BF_TEST_INSTANT_KILL=0
  BF_TEST_GOTO=-54.5,31,-54.5,45 BF_TEST_GO_AT=4.0 BF_TEST_SPRINT=4.0 BF_TEST_SKILL=2.0`) it
  reaches the panel 0.3 s after the hit, stays against it upright and slides down it over 1.4 s,
  as #35 take22's trooper did against the tower (there within ~0.1 s, down by ~1 s).
* **A blast on a corpse** (a Frag landing 2 m from Tex's body, `BF_TEST_GRENADE_TYPE=Frag
  BF_TEST_THROW=4.0,0.2` with the NPC at 0.15, -9): "ragdoll blast at 1.97 m of 8.0: 88.7 N s on
  the root"; the body, lying flat, jolts a few centimetres (friction 4 holds it); the loose guns
  fly.
* **Shots into a corpse** (the frozen Tex above, `BF_CAMERA_PITCH=-0.4
  BF_TEST_TRIGGER=1.5-1.7,4.0-4.2,5.0-5.2,6.0-6.2`): three hits "shot tex's body at ...", each
  bleeding where it struck and jolting the body.
* **The Energy grenade** (grenades.md's command without `BF_TEST_TOGGLE_KILL`): the first blast
  kills Flint and Hawk (both moving: their bodies start at 1.8 x their velocity), the second's
  bolts throw both corpses ("body thrown": Hawk's at 2.4 m/s, 1.7 up, landing 0.6 m off).
* **The knock-down** (`--test BF_MAP=flat BF_CHARACTER=0 BF_TEST_GOTO=0,8,0,8 BF_TEST_KNOCK=1.5`):
  Brutus is thrown back, down within 0.5 s and back up by 1.4 s.
* **The player's death, the death camera and the hand-over** (`BF_MAP=sdm_e34
  BF_TEST_GOTO=15.3,37,15.0,29 BF_TEST_DIE=1`): the guns drop at once, the body holds 0.35 s, falls
  against the rock and 2 m down the slope; the death camera circles it, the memory chip lies
  beside it, "BRUTUS" shows at 4.5 s and control passes at 5.0 s, as squad.md has them.
* **Other skeletons**: e34's level creatures that the squad killed (25 bones, a Spine3) built 19
  bodies in 13 pieces and fell like the rest.
* `cargo test --bin bf_play ragdoll`: the joint table's names, the angles' round trip, a body
  resting, sliding and spinning to a stop, the pendulum's period, rocking dying down, a chain
  settling, the movement check's three rules.

## Not done, fits and guesses

* **The collapse and the settling are slower than the takes'.** The Shrikes are down at
  +0.93-1.40 s (takes: 0.55-0.85) and the standing Tex at +1.47 (take05: 0.85); limbs and heads
  keep moving to +3-4.5 s (takes: still by 1.5-2), a head left at two of its limits off the floor
  sinking for seconds, and the Shrike Soldier kneels before it topples where take22's tumbles
  sideways at once. The game's numbers make limbs slow flywheels (1.125 kg m^2 on 4.5 kg) and
  damp little (0.5 a second), and a body woken with no push starts from rest; the takes' bodies
  lurch sideways as they start (take22: ~2.5-5 m/s at +0.40) and lie flat by about +1. Either the
  game's body is pushed as it wakes (the dropped gun's touch is the guess) or Ipion's contacts and
  constraints take more energy than this solver's. A live read of a dying body's core speeds
  (#280's shot list A1) would tell.
* Fits and readings: `REST_WAKE` (0.35 s); the CBT_DEAD handler's shove and up-kick off
  (`DEATH_SHOVE`, no take shows them); the joint axes' directions; the knock-down's start;
  friction 4 everywhere (not 6 on the "3" grounds); the movement check's radius (core+4, read as
  the body's reach) and its fixed 17 steps.
* The dropped guns and the loose objects don't touch the ragdoll, nor the living; the bodies of
  one ragdoll pass through each other.
* A running character's bodies start with the 9 x velocity impulse alone (the game's also keep
  the speeds their animation's motion controllers gave them). The running death's 3.4 m slide
  has no take to compare with (#280's shot list).
* Not compared: a body on a slope against a take, the FF2 deaths frame by frame, a death in a
  liquid.
* The bodies' removal (#151).
