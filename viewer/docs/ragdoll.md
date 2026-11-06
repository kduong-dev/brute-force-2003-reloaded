# The ragdoll (`src/bin/play_ragdoll.rs`, #259)

Every character's body at death, the squad's and the enemies' alike, and the squad's
knock-down, is the game's ragdoll: its bodies, joints, masses, damping, physics step, sleep and
pushes as #280's study and #259 read them from default.xbe (`todo/280-ragdoll/answer.md`,
`spec.md`; the rows in [code-map.md](code-map.md)), simulated by a small rigid-body solver of the
demo's own. It replaced the verlet point masses (#94) and #129's enemy branch of them. #307 gave
a death by a hit the game's shoves (a live read of the game's dying bodies,
`todo/307-ragdoll-death/notes.md`); #366 gave each kind of kill its own (a scenery area's,
a damage plate's, an Energy blast's; none for a Gas kill, a fall or a liquid's; a type-3 kill
vaporised, not ragdolled: the damage records read live, `todo/366-death-shoves/notes.md`). play.md's death paragraph, [enemies.md](enemies.md) "Death",
squad.md's knock-down and skills.md's Vengar charge point here.

## What the game does

* **The build** (FUN_00127290, one a frame from the world's queue: FUN_000dfff0, FUN_000d9a20):
  the guns drop (FUN_0014dfd0); the whole skeleton is disconnected from the animation (node flag
  0x20, FUN_00127160); each hit part's body (FUN_00114010, FUN_00114d00: an Ipion body from the
  part's collision surface, one for each of the type's `<collision>` bones, eleven on a biped) is
  beamed onto its bone (FUN_00169140) and its motion controller deleted; each ancestor of a hit
  part gets a 0.05 m ball (FUN_00114b80 -> FUN_002cce90), the character's node too (the root
  body, char+0x500). A bone below the lowest hit part of its chain gets nothing. The balls weigh
  and turn but meet nothing: FUN_00127290 clears their collision bits and, unlike a hit part's,
  never sets them again (0x1276a8-0x1276da).
* **Masses and inertia**: 9 kg for the root and for any body whose bone hangs from the
  character's node (Bip01), 4.5 kg for the rest (0x40cbe8 and half of it): 90 kg a biped. The
  inertia is set to 0.5 (the root) or 0.25 times that mass: isotropic, 4.5 and 1.125 kg m^2
  (FUN_00127290 writes the core's inertia, FUN_002bb410 its inverse). A 4.5 kg forearm turns like
  a 1.7 m rod: every limb is slow to swing.
* **Damping**: linear 0.95 and angular 0.5 a second (0x40cbe0, 0x40cbdc into the cores), taken
  each step as `v *= 1 - k dt` with the step's time (FUN_002baa70, from 0x2be2d6), 0.1 more once a
  body is calm (FUN_002ba9b0).
* **The world**: Ipion at 66 steps a second (FUN_002b4780), gravity 9.81 (FUN_0016a250); a
  ragdoll body's material against the ground gives friction 2 x 2 = 4 (2 x 3 = 6 on some
  grounds) and restitution 0.2 x 0.2 = 0.04 (the world's material table, 0x2bc0b0, 0x2bc0e0).
* **Joints** (FUN_00113120: a constraint per body with its parent's, by the child bone's name):
  the hip (Thigh) x -45..45, y -60..110, z -20..20 degrees; the knee (Calf) y -90..0, a one-way
  hinge; the HorseLink, the ankle, Spine1-3, Neck1, the head (-45..45 on each), the shoulder
  (UpperArm) x -55..55, y -40..100, z -45..80; the elbow (Forearm) y -100..0; the wrist; eight
  more cases of other skeletons. Every joint starts as a ball socket (FUN_002b90a0 with three
  fixed translations, type 1, and three free rotations, type 0); the listed bones' rotations get
  type 2, limited, and any other bone's (Bip01, the pelvis, the spine's base, the neck, the
  clavicles) type 1, fixed: the hips (the node, Bip01, the pelvis and the spine's base) and the
  chest (Spine2, the neck and the clavicles) are rigid pieces. **Correction:** #280's study read
  type 1 as free.
* **The joints' frame**: FUN_00127290 hands FUN_00113120 the parent bone's body as the
  constraint's reference object and the child's as the attached one (0x127738-0x127750), and
  FUN_002b9370 gives the reference side that body's own frame (no axis: FUN_002b6aa0's identity)
  at the child bone's origin (FUN_00039510), the attached side the same frame as it stands at the
  build. So the ranges are about the **parent bone's** x, y and z, and zero is the pose at death.
* **The start**: an impulse of 9 x the character's velocity on every body at its centre
  (FUN_002b3bc0), only when that velocity isn't 0. A running character's 9 kg bodies leave at its
  speed and the 4.5 kg ones at twice it: the whole body at 1.8 times it. A body that stood still
  gets no push of this kind, and nothing holds it: Ipion makes a new object out of the simulation
  (FUN_002b2f40: movement state 8), but FUN_00114b80 puts each ball into it at once
  (FUN_002b3c60), and the joints make the ragdoll one simulation unit. A death by a hit pushes
  it two game frames later (the shoves, below).
* **Sleep**: every 15-19 steps (a simulation unit's step, FUN_002be4c0, counts it down) the
  movement check looks at each body (FUN_002bb980): calm when it has gone at most 1 cm and its
  far end at most ~7 mm since its short reference, still once calm for more than 0.3 s (env+0x90);
  otherwise its short reference moves to where it is, and a body that in 4 s hasn't gone more than
  10 cm, nor its far end ~28 cm, from its long reference counts as still too. When every body of
  the ragdoll is still, it's frozen, its speeds zeroed (FUN_002be350 -> FUN_002bb0e0), until
  something pushes it.
* **Pushes** (FUN_000e00b0 queues, FUN_000d9a20 walks the queue, FUN_00127c60 applies; the
  death's, read live in #307: `todo/307-ragdoll-death/notes.md`, take01-take10):
  * *The request* (FUN_000e00b0: world, character, magnitude, damage type, factor; the point, the
    way and the body struck in registers). For a character whose ragdoll is queued or built
    (char+0x7d8 & 3) it goes into the world's queue: the first entry with a magnitude takes it in
    if it is for the same body (its point and way moved toward the new one's by its share of the
    two magnitudes, their magnitudes and factors added, the type of the larger), and is replaced
    by it for another body only when the new one is at least five times its magnitude and its
    factor; else the next entry is tried and the request becomes a new one. A death's two
    requests stay two entries.
  * *The walk* (FUN_000d9a20, 0xd9b54-0xd9c28, once a world update, 30 a second): the queue's
    first entry. A character not built yet is built (FUN_00127290) and the walk ends for the
    update; a built one is shoved (FUN_00127c60) and the entry is **removed whether it landed or
    not** (the test of the result jumps to the same removal, 0xd9bca-0xd9be3), the walk ending
    there if it landed with a factor over 0. A death's ragdoll is built the update after it; its
    bodies first show in the takes' per-frame reads at the second sample and the first shove at the
    fourth (66 ms later), the up-kick the update after it.
  * *The shove* (FUN_00127c60): nothing without a built ragdoll or a magnitude, **or while the root
    core's speed squared is over 100** (it opens with `FUN_0010ff10(core + 0xac) <= 100`, and
    FUN_0010ff10 returns x^2 + y^2 + z^2: a root over 10 m/s refuses the shove, which the walk
    then drops; #280 read it as 100 m/s). A factor under 3 of damage type 10 (an explosion) is
    made 3. Every body but the one struck gets min(0.5 x magnitude x factor, 75) x L N s along the
    way at its centre (only with a factor of 1 or more); the one struck min(8 x magnitude x factor,
    300) x L more at the point (the factor counting as 1 when it is 0); L = (1 - w) x 0.4 + 0.6,
    w the world's load (world+0xc58, FUN_000d9090: 0 to 1 by a count of live objects and
    explosions, #151). The seven takes' sums of the bodies' momentum jumps give L = 0.86-0.97.
  * *A death by a hit* (the CBT_DEAD handler, FUN_00115f90 case 0x4e, when the damage record's bit
    3, the shove gate, is set: FUN_00222f60's constructor sets it, the fall (FUN_0012b740) and the
    liquid (0x221610) clear it, and FUN_0022c4d0 gives a weapon blast's hit the inverse of
    the blast's flag 0x10, which is the Damage's h_0e373132, the Gas's alone: see "Which kills" below;
    a record that names no body keeps none, FUN_00115d50's store of the root, char+0x500,
    writes the 0 the unbuilt ragdoll has): two requests. **The hit's**: magnitude 2 x the damage
    the hit dealt (the record's +8 after the victim's step, skill, modifiers and damage-type
    factor: the whole of it, not what its hit points could take: L-Shot-75 80-92 for a 40-46
    hit, Tex's 14.3 for 7.1), the damage type (1 a bullet, 10 a blast), the weapon's push (the
    bullet's h_e39b994b: L-Shot-75 4, Bower 20 5, the Frag's blast 5) as the factor, at the point
    struck along the hit's way (not normalised) on the body struck (a blast's: the hit-part body
    its ball met first, take06's and take07's the same one, by its height an upper arm; the point
    the character's place, the way from the blast's centre to it: up and away for a Frag at the
    victim's feet, take06, away for one 2.5 m off, take07; the demo takes the root for the body).
    **The up-kick**: magnitude 1, type 0, factor 36, straight up on no body, 18 L N s a body
    (+3.2 m/s for the whole of it: take08's mean jump in the frame after it), not when the
    character is in the air (movement mode 2 or 3) or knocked down (motion type 25). A strong hit
    reaches both caps whatever the weapon or the victim (a rifle's, a shotgun shell's and a blast's
    all throw the 18 bodies to a mean of 16 m/s along the shot; a Heavy and a Soldier took the
    same, take01 against take02), and the root is then over 10 m/s, so the up-kick behind it is
    refused and dropped. A weak hit's throw scales (take08: 8.9 m/s), the root stays under 10
    m/s and its up-kick lands.
  * *A blast on a corpse*: FUN_00224a90 gathers a character through its root body once its
    ragdoll is built (0x224e41), applies the damage to it (FUN_00224320) and the damage handler
    (0x125600, 0x125853) asks for a shove of a character already dead whose ragdoll is queued or
    built: the record's body (the root), point (its place) and way (away from the blast), magnitude
    +8 as the factors left it, type, **factor 0**, so a blast's type 10 makes it 3 and **every body
    of the list is thrown** (take10: magnitude 44.6, a mean of 15.5 m/s at the jump; #280 had the
    root alone). Then, when the weapon pushes (1 or more), FUN_00224a90's own push of the root body:
    1.5 x (1.2 - d / r)^2 x the blast's damage N s away from the blast, when its ball meets one of
    the body's hit parts with a clear ray from its centre (the same gathering as the living's).
    A bullet through a corpse never asks (take10's two shells: no request, the body at 0.00-0.02
    m/s; whether they struck it isn't known).
  * The Vengar's charge queues magnitude 75 and factor 2 along his velocity and 2 up, on no body:
    75 N s on every body, ~15 m/s for the whole body.
  * *Which kills the game throws* (#366: the damage records read live at 0x11602d, `todo/366-death-shoves/notes.md`
    take01-06, e34 on STANDARD, a frozen Shrike Soldier killed; the other rows from the code,
    `leads.md`). What a kill does is what its record says: who built it sets the type, the push, the
    gate and the body.

    | the kill | record (type, push, gate, body) | the request, and what comes of it |
    |---|---|---|
    | a rifle, a shotgun shell, a missile hit (#307) | the weapon's damage type and push, open, **the bone body struck** | the hit, then the up-kick as its own entry: every body 75 L, the one struck 300 L more: ~16 m/s mean along the shot |
    | a scenery area, Type 10 (the e34 barrels, the racks): take01, a missile rack | 10, 1.750 (L x the area's 2), open, **no body (0)**, a way from the object's origin to the root body, (-0.029, 0.320, 0.947) | the hit's request has no body, so the up-kick **merges into it**: one request, magnitude 2m + 1 = 16.7, factor 37.75, the way tilted a little to up (-0.027, 0.360, 0.890): only the all-body share, at the cap: **12.7 m/s mean at 20 degrees up, 8.5 m** |
    | a scenery area, Type 3 (the radiation barrels): take02 | 3, 1.70, open, no body | **never ragdolled**: the damage handler clears the byte 0x3c05f8 around a type-3 record (0x1257a7) so FUN_00115d50 queues no ragdoll (0x115e13): no REQUEST, no BUILD, and FUN_001252b0 vaporises the body (the removal timer -99): a green burst and no corpse (#147) |
    | an Energy blast: take03, take05 | 6, **2.0**, open (0xfb), a bone body (7f5e8400), away from the blast's centre (take03's (0.098, 0.911, -0.400): the blast sat low beside him) | the hit (magnitude 2 x 77.8, factor 2.0) and the up-kick as separate entries, as a Frag's: **mean 15.4 m/s, root 12.4, 65 degrees up**, the body 5 m up and 6 m away; the up-kick refused at 10 m/s |
    | a Gas cloud's tick: take04 | 4, 2.0, **closed (0xf3)** | **no request at all**, not even the up-kick: the body collapses where it stood, falling from the first frame (mean 0.34, 0.47, 0.75, 0.97 ... m/s a frame after the build), the root 0.6 m off after 3 s |
    | a fall, a liquid (code) | 0, 1.0 and 5, 0.2, **closed** | nothing, as the Gas |
    | a damage plate, a bare deal, a destroyer (code; the plate's not filmed) | the plate's type, 1.0, open, no body, **no way (0, 0, 0)** | one body-less request merged with the up-kick, its way (0, 1 / (1 + m), 0): **straight up**, 0.5 x (2m + 1) x 37 at most 75 per body |
    | a corpse in an Energy blast: take06 | the blast's request: 6, magnitude 58.3, **factor 0**, the root body, away from the blast | no all-body share (it needs a factor of 1): the hit-point share min(8 x 58.3, 300) L on the root (and by the code the explosion's own push of the root, which the take can't tell apart): **root 3.3 m/s, mean 2.4, the body slid 0.45 m and lay still**; a Frag's (type 10) raises the factor to 3 and throws every body |

    L = (1 - w) x 0.4 + 0.6, w the world's load (world+0xc58): 0.875 at take01's 0.313.
* **Where the corpse is**: its root body (0x1271a0 writes it into the character's node after
  each step): the death camera, the blood pool and the memory chip go by it.
* **What it meets** (#298): the terrain and the placed objects. The level's invisible blockers
  and hinters meet a character's body alone: Ipion's pair table at 0x3c01e8 pairs PID_BLOCKER and
  PID_HINTER with PID_CHARACTER_PHANTOM and nothing else, and a ragdoll's bodies are PID_BONE, so
  a corpse goes through them ([maps.md](maps.md), "Only a character meets them").

## The demo's model

* **Bodies and pieces.** The game's bodies are made as the game makes them (`Ragdoll::new`), each
  with its mass, inertia and centre (the part's mass centre, `Character::part_mass_centre`), then
  the two sides of each rigid joint are put together into one solver body (`Body`): their masses
  summed, their inertias with them (each its own and its offset's, isotropic in the mean,
  2/3 m r^2). A biped's 18 bodies come out as 12 pieces with 11 limited joints, a Shrike's too;
  a 25-bone creature with a Spine3 as 19 bodies in 13 pieces. A piece of balls alone (a Spine1)
  has no contact points.
* **Joints.** A joint holds its two bodies together at the child bone's origin and keeps the
  child's turn since death within FUN_00113120's ranges, measured about the parent bone's axes
  (`Joint`): its twist about x and its swing's parts about y and z. How Ipion splits a turn into
  the three angles isn't traced; this twist-and-swing reading bends knees and elbows the way they
  bend and starts every joint inside its range.
* **Bones without a body** (the feet, toes, hands and fingers below the lowest hit part, a
  Shrike's HorseLink, and every bone of the face, the hair and the props) keep their pose at
  death under their parent (the face's is what the face controller last wrote, the blink's lids
  at the point in the clip they were: [faces.md](faces.md)), and a bone with a body takes its body's turn at its own place under
  its parent: no bone stretches or slides against its neighbours, so faces and limbs don't morph
  as the verlet body's did.
* **Contacts.** Each body's points (its hit parts' surfaces' corners, at most 26 a part) against
  the level's faces round it (`Arena::faces_near`) that the body is in front of, over the face
  (2 cm past its edges counts); on the flat test floor without scenery, a plane where the feet
  were. A point already in the ground at the start comes out at most 0.5 m/s (a Shrike's calf
  surface starts 3 cm under the floor). Friction 4 and restitution 0.04 everywhere. Bodies of
  one ragdoll don't meet each other, nor the living, the loose objects or the dropped guns (spec
  S2.5); the balls meet nothing, shots included.
* **Blockers and hinters** (#298, `Ragdoll::as_body`, `arena::Body`): a corpse's ragdoll is a
  thing's body, which `faces_near` and the floor under it (`floor_below`) take without the
  level's blockers and hinters, so it falls and rolls through them; a knocked-down character's is
  a character's, which keeps them (the demo stands the character up where its pelvis lies, so it
  mustn't end the far side of a wall the living can't cross; the game's knock-down is a clip
  that moves the character itself). One that dies while still limp from a knock-down dies where
  it lies: its ragdoll goes on as the corpse's (the hit's `hurt` keeps the knock for it) and
  becomes a thing's body; one already getting up starts a new ragdoll from its pose.
* **No hold** (#366). A body that stood still (under 5 cm/s) is simulated from the build, as the
  game's is (FUN_00114b80 puts each ball into the simulation at once). #259 held it 0.2 s
  (`REST_WAKE`), a fit to the four takes' marks of a 0.35-0.45 s stand; #307 read that as the hit
  logs' breakpoints stopping the machine (the kills' bodies move from the build); #366's take04, a
  Gas kill that asks for no shove, settles it for the body left to itself: the bodies fall from
  the first sampled frame, and the demo's mean-speed series (0.24, 0.49, 0.74, 0.99, 1.17, 1.40,
  1.60 ... m/s) is the take's (0.34, 0.47, 0.75, 0.97, 1.15, 1.31, 1.46 ...) to within a sample.
  A kill's shove comes at 66 ms. #259's bodies' "landing within 1 m" was a thrown body seen along
  its way (the camera 7 m in line with the shot; the frames read as a throw: take22's corpse ends
  by the barrel at the gate's foot, behind where the Soldier stood, and #28 take05's Tex is off
  the ground and tumbling 0.4-0.7 s after the hit).
* **The death's shoves** (#307, `death_shoves`, `request`, `Ragdoll::walk`, `Ragdoll::shove`): a
  kill by a shot (`land_ai_shot`, `land_player_shot`, the bolts'), a missile's direct hit or a
  blast (play_grenade.rs `hurt_by_blast`, play_missile.rs) puts the CBT_DEAD handler's two
  requests into the character's queue, `Player::shoves` (merged as FUN_000e00b0 does), from the
  hit's record: `Hit::amount` as the damage (the whole of it, 2 x as the magnitude),
  `Hit::damage_type`, the weapon's `WeaponDef::push` (bullet h_e39b994b) as the factor, the point and
  way the shot struck by, and the part struck (`Hit::bone`; a weapon hit the demo found no part
  for stands in with the root, `Target::from`: the game's weapon records always name their body; the
  up-kick is on no body); the up-kick unless the character is in the air or knocked down. The
  ragdoll's `walk` takes the queue a game frame (1/30 s) at a time from the second after it's built,
  as FUN_000d9a20 does, each entry applied (`shove`: FUN_00127c60's caps, the explosion factor,
  the gate on the root's speed squared) and dropped. The world's load L is a constant
  `SHOVE_LOAD`, 0.9 (a fit to the takes' momentum jumps, 0.86-0.97: the demo has no count of the
  world's live objects). The Vengar's charge queues its knock-back the same way.
  **Each kill is its record's** (#366, the table above): a scenery area's kill
  (`play_scenery.rs` `area_damage`) is body-less (`Target::None`) with the area's push
  (`area_push`: L x h_ed582b3c) and the way from the object's origin to the victim's root body
  (`root_point`), so the up-kick merges into it and the throw is the all-body cap; a damage
  plate's (`play_damage_plate.rs`) is body-less with no way, so it hops straight up; an Energy
  blast's bolt kill (`play_energy.rs`) is a Frag's, type 6 and push 2 on the root as the stand-in
  for the bone body its ball met; a type-3 kill is **vaporised** (`Player::vaporised`, from
  `take_hit`): no ragdoll built, nothing queued, the model hidden (`hide_vaporised`; the effect and
  the chain are #147's); a Gas kill, a fall and a liquid's ask for none, and the body just falls.
  `BF_TEST_DIE`'s death has no record, so none either.
* **The movement check** runs every 17 steps (the game's 15-19 at random) with the game's
  thresholds (`movement_check`), and its calm state adds the game's 0.1 to both dampings.
* **The knock-down** (the squad's own: the game's is a clip, #129) starts the same body with the
  demo's old throw, the hit's push x 1.6 and 2.5 m/s down, both by height (`Start::Knock`); the
  character gets up from where its root lies after 0.7 s.
* **Shots and the Energy grenade.** A shot through a body on the ground (dead or knocked down)
  brings the hit part it meets first up to 3 m/s along the shot at that point, no faster, and it
  bleeds there (spec S6.5: the demo's, no such code was found; #129 take14's lying Heavy shifts
  as it's killed). The Energy grenade's bolt on a corpse is the blast's own request, not a throw
  of the demo's (#366: `grenade::blast_on_corpse`, type 6 and factor 0: take06's root 3.3 m/s, no
  all-body share); on a living squad member lying knocked down it still throws the body up and out
  at 3 m/s, every body brought up to it (`Ragdoll::fling`, the demo's). A blast on a corpse (`blast_corpse`)
  asks the damage handler's shove, magnitude the damage a living body would take at the root's
  distance (a grenade's rule is the demo's until #112, a missile's the game's share), factor 0
  (so an explosion's 3: every body thrown), the root struck, away from the blast; and, when the
  weapon pushes, the explosion's own push of the root (`Ragdoll::blast`). A grenade's blast does
  so with no cover test, as it hurts the living; a missile's must see one of the hit parts, as the
  living's must.

## The solver

Extended position-based dynamics (Müller et al., "Detailed Rigid Body Simulation with Extended
Position Based Dynamics", 2020), 16 substeps a 1/66 s step. Each substep moves the bodies by their
velocities and gravity; brings each joint back within its ranges, then pulls its two points
together; pushes each contact point out of its face; then takes the velocities from the moves: a
point that struck its face leaves it at 0.04 of the speed it came in at (none under 2 g dt), and
its sliding is taken away up to 4 x its push. Damping and the movement check come once a step, as
Ipion's. The game's isotropic inertia makes every turn's share a plain one.

* **A joint's stops** (`solve_joint`): the smallest turn that brings the angles past their stops
  back to them and leaves the others as they are (to first order: each angle's rate of change
  with a small turn of the child about each world axis, `RATE_STEP`), at most 0.05 rad a
  substep (`MAX_STOP_TURN`). The first round rebuilt the child's turn from the clamped angles
  instead: that turn wasn't square to the stops, so it fed energy into the free axes (a body held
  at two stops crept along them for seconds), and near the twist-and-swing split's singular turns
  it leapt (the wall pop below). Then the points: the stop's turn moves the joint's point, and
  the point's correction closes it again.
* 16 substeps hold a 90 kg body's joints to under a millimetre (`BF_RAGDOLL_LOG`'s joint gap); a
  pendulum of the game's limb swings at its 2.42 s period (a unit test): the solver adds no
  stiffness or drag of its own, and a body sliding along its stops loses energy every step.

A crate was weighed: rapier or avian would want the level's triangles mirrored into their own
world and kept in step with the doors and breakables `Arena` already tracks, for a dozen bodies a
character, and neither is Ipion's solver (a linear system over a ragdoll's joints each step,
FUN_002db0a0 / FUN_002db5a0) either. The demo's solver is one module with no new dependency,
takes the game's inertias, damping and sleep rule as they are, and reads the arena directly.

## Test hooks

* `BF_RAGDOLL_LOG=1`: at the build, the bodies (their parts, mass, inertia, contact points,
  triangles and balls, how high the lowest point is), the joints (their ranges, where a turn
  about the joint's -y and -z takes the child's centre) and the start; then a line a frame for
  3 s and every half second after (`BF_RAGDOLL_LOG=all`: every frame throughout), with the root's
  place and speed, the mean speed and mean velocity of the game's 18 bodies and the highest
  core (#307: what the takes' per-frame reads log, `todo/307-ragdoll-death/*.rd.jsonl`), the
  fastest body and spin, the trunk's lowest point over the floor and the floor under the root, the
  worst joint gap, the contacts, how many bodies are calm and still, asleep; each death's shoves
  asked, each shove (and each one the root's speed refused) and blast, each shot into a body,
  and "died knocked down" when a knocked-down body becomes a corpse (play.rs).
* `BF_RAGDOLL_CORES=1` (with `BF_RAGDOLL_LOG`): each log line is preceded by every one of the game's
  bodies' place and velocity, the takes' per-body reads.
* `BF_RAGDOLL_DRAW=1`: every ragdoll's bodies (green surfaces, yellow balls, the root's piece red)
  and joints (blue).
* The kill types (#366: a scenery area's, a radiation barrel's, an Energy's, a Gas, an Energy on a
  corpse, a damage plate's): no hook of their own, the existing ones set them up, each case's
  command under "Verified" (`BF_TEST_HIT` / `BF_TEST_HIT_NEAR` for a rack or a barrel,
  `BF_TEST_GRENADE_TYPE` with `BF_TEST_THROW` or `BF_TEST_DROP`, `BF_TEST_NPC` with its `y` to stand
  it on the ground, `BF_TEST_NPC_HP`, `BF_TEST_HEALTH` for the controlled character).
* Deaths and knocks to watch: `BF_TEST_DIE=<s>` ([play.md](play.md)), `BF_TEST_KNOCK=<s>`
  (with `BF_TEST_FULL_SQUAD=1` for a member other than Tex on the test map), a frozen enemy shot
  (`BF_TEST_NPC=...,frozen,...` with `BF_TEST_TRIGGER`, [enemies.md](enemies.md)),
  `BF_TEST_FREECAM` to look ([testtools.md](testtools.md); it holds the player still while it's
  on, so for a running death start it after the death).

## Verified

Captures at 15 fps with `BF_MUTE=1 BF_BACKGROUND=1` (a log-only run needs `BF_CAPTURE` too, with
`BF_CAPTURE_EVERY=1000`, to be bounded) and `BF_RAGDOLL_LOG`.

### The kills against #307's per-frame reads (the shoves)

The demo's cases are set up as the takes were (the spots, the weapons, the victim's hit points
set so one hit kills): `--test BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GO_AT=100
BF_RAGDOLL_LOG=all BF_CAMERA_PITCH=-0.12`, the controlled character at the first point of
`BF_TEST_GOTO` facing its second, `BF_TEST_NPC_HP=1.0,1,<hp>`, `BF_TEST_TRIGGER=1.5-1.7` holding
its trigger:

* take01 (R1) `BF_MAP=e34 BF_CHARACTER=1 BF_TEST_WEAPON=L-Shot-75 BF_TEST_GOTO=60.2,-10.9,58.31,-4.16
  BF_TEST_NPC="0.3,h_e9a41520,enemy,frozen,58.31,-4.16"`, hp 30; take02 (R2) the same with
  `h_1c5780f8` (the Heavy) at 58.32, hp 20.
* take08 (R5) Flint's L-Shot-75 on Tex: `BF_MAP=e34 BF_CHARACTER=1 BF_TEST_WEAPON=L-Shot-75
  BF_TEST_GOTO=-90.5,-82.6,-83.5,-82.6 BF_TEST_NPC="0.3,tex,friend,frozen,-83.5,-82.6"`, hp 5 (a
  hit of 8.0, the take's 7.1 after the squad's factors: the weak one).
* take09 (S1) `BF_MAP=e40 BF_CHARACTER=0 BF_TEST_WEAPON="Bower 20"
  BF_TEST_GOTO=-0.5,166.98,-4.17,165.4 BF_TEST_NPC="0.3,shrike soldier,enemy,frozen,-4.17,165.40"
  BF_TEST_TRIGGER=1.5-1.6`, hp 8.
* take06 (B1) and take07 (B2) `BF_MAP=e40 BF_CHARACTER=0 BF_TEST_GRENADE_TYPE=Frag
  BF_TEST_GOTO=-1.97,169.30,-20.2,160.6 BF_TEST_THROW=1.0,0.2`: the Frag goes off at
  (-29.84, 156.11); the NPC (`shrike soldier`, hp 5) at (-28.84, 156.11) (1.4 m from it, the
  blast's way 44 degrees up: take06's was 41) or (-32.34, 156.11) (2.7 m, 21 degrees up).
* take10 (S2's blast) S1's with `BF_CAMERA_PITCH=-0.35 BF_TEST_NPC_HP=1.0,1,4
  BF_TEST_GRENADE_TYPE=Frag BF_TEST_THROW=5.0,0.1`: the kill, then the Frag 2.0 m from the corpse.

Times are after the first shove (the demo's 66 ms after the death, the takes' fourth sample);
"mean" is the mean of the 18 bodies' speeds, as the takes' `rd.jsonl` rows give it:

| case | mean speed, m/s, at +0, 0.1, 0.2, 0.3, 0.5, 0.8, 1.0, 1.5 s | thrown | mean under 0.5 / 0.1 m/s from | top core within 0.25 m of rest from |
|---|---|---|---|---|
| R1 Soldier, L-Shot-75 | demo 17.2 13.6 12.6 9.8 5.7 3.2 1.9 0.3; take 15.9 14.5 9.7 6.7 4.2 3.2 2.5 1.8 | 7.6 m; take 5.8 m (log to 1.3 s) | demo +2.5 / +3.1 | demo +2.1; take +0.7 (log to 1.3 s) |
| R2 Heavy, L-Shot-75 | demo 17.2 13.6 12.7 8.6 7.1 2.2 1.0 1.2; take 16.1 13.7 12.3 3.8 3.3 1.7 2.2 2.3 | 7.1 m; take 5.2 m (to 1.2 s) | demo +1.9 / +2.5 | demo +1.9 |
| R5 Tex, weak hit | demo 8.2 8.0 7.4 6.5 5.8 4.8 1.9 0.2; take 8.9 8.4 7.4 6.6 5.7 5.6 2.2 0.4 | 5.5 m; take 5.8 m | demo +1.5 / +2.6; take +1.5 / +3.1 | demo +1.4; take +1.8 |
| S1 Soldier, Bower 20, 4 m | demo 17.2 14.8 13.2 11.4 4.2 2.8 2.8 0.2; take 16.3 13.4 12.0 11.1 6.1 2.6 2.0 0.9 | 7.9 m; take 7.4 m | demo +1.4 / +1.9; take +1.6 / +2.2 | demo +1.3; take +1.4 |
| B1 Soldier, Frag at his feet | demo 16.1 13.8 12.4 10.1 7.6 5.7 5.2 6.4; take 16.9 14.0 12.6 10.8 8.2 5.8 5.3 5.5 | 10.6 m; take 9.7 m (to 1.3 s, still at 5.5 m/s) | not in the take's log | |
| B2 Soldier, Frag 2.7 m off | demo 16.1 14.2 13.1 11.2 9.3 4.4 3.6 0.8; take 17.5 14.8 13.9 12.5 10.6 3.5 3.4 2.3 | 8.9 m (onto a dune); take 10.3 m (to 1.4 s) | not in the take's log | |
| S2 corpse, Frag 2.0 m off | demo 16.3 at +0, then 10.2 8.0 6.6 3.5 2.2 1.6 at +0.07 0.2 0.33 0.5 0.65 1.0; take 15.5 at +0, 13.9 +0.1, 9.3 +0.2, 2.9 +0.27 (stopped dead) | 4.8 m; take 3.2 m (stopped by something the demo lacks) | at rest by +1.5 s; take +1.5 s | |

The takes' chatter logs show the up-kick refused at its first try (`al 0`, the root over 10 m/s)
in R1, R2, B1, B2 and S1 and applied in R5, and the demo's does the same (`ragdoll shove refused`
in its log). The same S1 at 30 and 60 fps (`BF_CAPTURE_FPS`) gives the same series to within a
frame's shift (the landing at +0.5 s reads 4.2, 8.1 and 8.7 m/s at 15, 30 and 60 fps, the frames
falling either side of it) and the distance 7.9, 8.0 and 8.1 m. The first-frame root speed (14.9
m/s in take01) is 10 m/s in the demo straight after the shove and 13.5 a frame on: the demo's
solver shares the chest's push through the joints over a few substeps where Ipion solves them
together. A velocity-level solve of the joints after each shove (Gauss-Seidel on the joints'
relative point velocities) gave the first frame (14.9) but not a better tail: S1 went 9.2 m
against 7.9 without it (the take's 7.4), and the RMS of the mean-speed series against the takes rose
in three of six cases and fell in three, so it is not in.

**What doesn't match.** R1 and R2 go a third further than the takes on e34's sand: their feet catch
at +0.13-0.2 s (take01's calves both stop dead at z = -1.9..-2.0 at 13 m/s, the Heavy's chest at
z = 0), where the demo's catch in the first frame and swing up (their soles start at the floor:
`BF_RAGDOLL_CORES`; starting the whole body 5 or 10 cm higher changed neither throw, 7.2-7.9 m). The
arena's faces there are the terrain's 4 m triangles with no step or object,
so what the game's bodies met isn't known (not the level's blockers: R2 with the corpse's body made
a character's is the same run). S1 and B1, on flat ground, match to within 10% of the mean speed
all along. B2's and S2's lie on other ground (a dune; something that stopped the corpse). The demo's
bodies come to rest 0.2-0.5 s sooner than the takes' (the mean speed under 0.1 m/s from +1.9-3.1 s,
the takes' +2.2-3.1 s); neither freezes its ragdoll within the logs (the takes' bodies read the
movement state 1 for ~3 s and some to 5 s; the demo's freeze between 4 and 10 s or not by 11 s,
a forearm creeping at ~0.5 rad/s).

### The kill types against #366's reads

`todo/366-death-shoves/` (take01-take06: the rack, the radiation barrel, an Energy kill, a Gas kill, a
second Energy kill, an Energy blast on a corpse, each with the damage record read at 0x11602d). The demo's
cases are set up as the takes were, `BF_MAP=e34 BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GO_AT=100
BF_RAGDOLL_LOG=all BF_COMBAT_LOG=1`, a frozen Shrike Soldier (`h_e9a41520`, `BF_TEST_NPC=0.3,...,enemy,frozen,x,z,y`: the
`y` finds the ground under it, 30.5 on the sand) with `BF_TEST_NPC_HP=1.0,1,5`, captures at 30 fps
(`BF_CAPTURE_FPS=30`: the takes' game frame), compared as the table above, in seconds after the first shove:

* **A, a missile rack's kill (take01)**: `BF_CHARACTER=1 BF_TEST_GOTO=-39,22.5,-39,22.5
  BF_TEST_NPC=0.3,h_e9a41520,enemy,frozen,-31.97,25.44,30.5 BF_TEST_HIT=2.0
  BF_TEST_HIT_NEAR=-31.86,21.87`: the Soldier at the take's place. The demo's log asks for "magnitude 18.7 type 10
  factor 1.8 ... along [-0.03, 0.31, 0.95] on None, then the up-kick" and lands "magnitude 19.7 type 10
  factor 37.8 along [-0.03, 0.36, 0.93]: 67.5 N s on every body": one request, as the take's (record
  15.7, 10, 1.750, (-0.029, 0.320, 0.947); merged 16.705, 37.750, (-0.027, 0.360, 0.890)). The mean speed
  at +0, 0.3, 0.5, 1.0 s: demo 13.9 9.7 7.9 6.5, take 12.7 8.8 7.3 5.9 (+9%: the fit's L is 0.9 where
  take01's is 0.875, and the solver's first sample); the throw's elevation 21 and 20 degrees; the top
  core's rise 0.83 and 0.76 m; the body went 9.3 and 8.5 m. Before the change a rack kill asked for
  no shove, and the body folded where it stood.
* **B, a radiation barrel's kill (take02)**: the same with the Soldier at (-31.03, 15.83) and the
  barrel h_f8234758 (`BF_TEST_HIT_NEAR=-31.05,17.52`): the Soldier dies at 2.2 s of the type-3 cloud, and the
  log has no ragdoll line, no shove (no "ragdoll" line at all), his model hidden from that frame
  (the free camera at (-38, 31, 17), `BF_TEST_FREECAM=1.5,-38,31,17,-90,-8`: the frames 60-65 show
  him standing, from 66 the barrel's burst and no corpse). On sdm_e34 the
  controlled Tex killed the same way (`BF_MAP=sdm_e34 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6
  BF_TEST_HIT=2.0 BF_TEST_HEALTH=20`): no corpse, the death camera circles the empty place and her rifle on the
  ground. The take's burst (h_eecb39f9's bio effect) and the carrier's chain are #147's, not here.
* **C, an Energy kill (take03)**: `BF_CHARACTER=3 BF_TEST_GOTO=60.2,-7.1,59.7,12.7
  BF_TEST_GRENADE_TYPE=Energy BF_TEST_THROW=1.0,0.5`, the Soldier 0.45 m from the blast at
  (60.01, 29.46, 11.09), the way of take03's "the blast sat low beside him": "magnitude 259.7 type 6 factor 2.0
  along [0.41, 0.91, 0.01] on Root, then the up-kick" (the take's record: 155.7, 6, 2.0, (0.098, 0.911,
  -0.400), a bone body), 67.5 N s on every body and 337.5 on the root, the up-kick refused at 16 m/s as take03's
  second APPLY (`al 0`). Mean speed at +0, 0.1, 0.3, 0.5, 1.0 s: demo 16.1 14.3 10.5 7.2 2.6, take 15.6
  12.6 9.0 6.1 2.7; elevation 66 and 65 degrees; the root went 6.7 and 6.2 m; the top core rose 6.2
  and 5.2 m (the vertical speed's 6% over is squared by the apex).
* **D, a Gas kill (take04)**: `BF_CHARACTER=3 BF_TEST_GOTO=60.2,-10.9,59.7,-4 BF_TEST_GRENADE_TYPE=Gas
  BF_TEST_DROP=2.0`, the Soldier at 1 m with `BF_TEST_NPC_HP=1.0,1,1`: no "death shoves" line, no shove, and the body
  falls from the first frame: mean speed 0.24 0.49 0.74 0.99 1.17 1.40 1.60 1.81 2.01 2.18 m/s a frame
  apart, the take's 0.34 0.47 0.75 0.97 1.15 1.31 1.46 1.67 1.83 2.00; it lands at +0.33 s
  (the take's +0.43) and lies after 2 s, the root 0.8 m off (take 0.6). With the old hold of 0.2 s the
  fall started 0.2 s late.
* **E, an Energy blast on a corpse (take06)**: the Soldier shot dead by an L-Shot-75 (R1's recipe, 5 m
  along the shot), an Energy thrown at 7 s (`BF_TEST_THROW=6.0,0.65`) onto 4.8 m from the body: "ragdoll shove:
  magnitude 63.2 type 6 factor 0.0 ...: 0.0 N s on every body, 270.0 N s on the part struck" and "ragdoll blast at
  4.83 m of 9.0: 64.3 N s on the root": the take's request is (58.3 at 3.9 m, 6, 0.0, the root). Mean speed
  2.3 at the kick (the take 2.4, root 3.3 after the first sample's solver sharing, the demo's 2.6), 1.4 0.6 0.25 at
  +0.1 0.2 0.5 s (the take 1.0 0.7 0.6), the root 0.3 m off (take 0.34): a nudge, as the take's. A Frag
  still throws every body (S2's row above).
* **F, a damage plate's kill** (not filmed): the tutorial's plate (`BF_MAP=tutorial BF_TEST_GOTO=37,67.5,37,67.5,32
  BF_TEST_PANEL="a,1;a,1;a,1;a,1" BF_TEST_HEALTH=1.5`), Tex killed at 0.3 s by the 0.40 HP frames:
  "magnitude 0.8 type 1 factor 1.0 ... along [0.00, 0.00, 0.00] on None, then the up-kick", landed as
  "magnitude 1.8 type 0 factor 37.0 along [0.00, 1.00, 0.00]: 30.0 N s on every body": straight up at a mean
  6.5 m/s, a hop of 1.4 m, then down. The shape is take03 of #307's trainer kill (the body-less zero-way
  request, 12 m/s straight up); the plate's own record is the code's.
* **Unchanged**: #307's R1 (a rifle kill, `BF_TEST_NPC_HP=1.0,1,30`) after the hold's removal: mean speed at
  +0, 0.1, 0.2, 0.3, 0.5, 1.0 s 17.2 13.8 12.6 10.5 4.3 2.5 (the table above's 17.2 13.6 12.6 9.8 5.7 1.9), the
  body went 6.5 m (take 5.8); a Frag 1.6 m from a Soldier (B1's recipe, the Soldier put 1.3 m past the
  blast, `BF_MAP=e40 BF_CHARACTER=0 BF_TEST_GOTO=-1.97,169.30,-20.2,160.6 BF_TEST_THROW=1.0,0.2`: the
  throw of #161 lands it at (-20.6, 160.65), not the old recipe's) at +0, 0.1, 0.3, 0.5, 1.0 s 15.8 14.0 10.7
  8.4 6.1 (take06 of #307 16.9 14.0 10.8 8.2 5.3).

### The old table's cases, as they are now

"Moves": the root body 3 cm off where it started; "down": the trunk's lowest point (the root's piece
and the chest's) first within 5 cm of the floor; "quiet": the last frame with a body faster than
0.3 m/s or turning faster than 1 rad/s; times after the hit (a log line every 1/15 s for 3 s, then
every half second). #259's first three rows were fitted to takes 17-22's marks (a stand of
0.35-0.45 s, the trunk down at +0.55-0.85 s, still by 1.5-2 s, "landing within 1 m"); with the
shoves those kills are thrown, and the takes' marks are best read as theirs alone: the camera looked
along the shot and their breakpoints stopped the game at the hit. The player's running death
(`BF_TEST_DIE`: no hit, no shoves) is as before.

| case | moves | down | quiet / frozen | root moved |
|---|---|---|---|---|
| Shrike Soldier, frozen, shot at take22's spot, the gate 3.6 m behind (`--test BF_MAP=e34 BF_NO_SQUAD=1 BF_CHARACTER=1 BF_TEST_GOTO=-53.6,30.8,-53.71,31.49 BF_CAMERA_PITCH=-0.12 BF_TEST_NPC=0.3,h_e9a41520,enemy,frozen,-54.6,37.8 BF_TEST_TRIGGER=1.5-1.7`) | 0.13 | 0.93 | 2.87 / 6.5 s | 1.74 m (against the gate) |
| Shrike Heavy, the same (`h_1c5780f8`, `BF_TEST_INSTANT_KILL=0 BF_TEST_NPC_HP=1.0,1,20`) | 0.13 | 0.73 | 2.67 / not by 9.5 s | 2.91 m |
| Shrike Heavy, enemies.md's "Death" command | 0.13 | 1.13 | 3.07 / not by 8.1 s | 3.95 m |
| Tex, an NPC, frozen, shot on the flat floor (`--test BF_MAP=flat BF_NO_SQUAD=1 BF_CHARACTER=1 BF_TEST_GOTO=0,8,0,7.3 BF_TEST_NPC=0.3,tex,enemy,frozen,0.15,3 BF_TEST_TRIGGER=1.5-1.7`) | 0.13 | 0.60 | 3.07 / 6.5 s | 8.66 m |
| Tex, the player, running at 6.9 m/s (`--test BF_MAP=flat BF_TEST_GOTO=0,20,0,-20 BF_TEST_DIE=2.2 BF_TEST_FREECAM=2.25,5.5,0.6,11,90,-10`) | 0.13 | 1.47 | 3.07 / 6.5 s | 6.35 m on |

* **The hold's fit** (#259: frozen enemies shot dead, released after 0, 0.2 and 0.35 s): the marks
  it was fitted to are the paused takes' (above); the Gas take of #366 shows the bodies fall from
  the first frame, so the hold is gone (the 0.2 s a death with no hit still held it).
* **No morphing** (main's build against this one, close-ups of Tex and the Shrikes through the
  fall and at rest, rechecked after the fix round): the verlet body's face and limbs smeared and
  stretched as its points were pulled about; now every bone keeps its shape and the face stays
  as it was at death.
* **Bodies land and stay**: a death with no hit lands within 0.3-1.0 m of where it stood; a kill
  throws the body 5-11 m along the shot (the table above), and none bounces or slides on once it is
  down. None went through the ground or a wall in these runs: over the table's cases and the
  three on e34's gate below the worst joint gap was 6 mm (B2, 0.8 s after the shove), the lowest
  trunk 0.017-0.05 m under the floor under its centre on e34's slope and 0.09 m on B2's dune
  (a trunk's low point on a slope, not a body through it: the contacts are gathered by what a body
  moves in a step, `reach`), a Soldier and a Heavy thrown at the gate 3.6 m behind them (take22's
  spot) stopped at it, rebounded 1 m and lay down, no NaN. The fastest body is the one struck for
  the first frame (a forearm or an upper arm at 75 m/s for 337 N s on 4.5 kg, then 17-30 m/s as the
  joints take it up) and the fastest spin 41 rad/s (the head, after a Frag on a corpse; the takes'
  Frag kills 45-57).
* **The Vengar's charge into a wall** (e34's gate panel 3.8 m behind: `--test BF_MAP=e34
  BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_NPC=0.5,tex,enemy,frozen,-54.5,37.5 BF_TEST_NPC_HP=1.0,1,125
  BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=-54.5,31,-54.5,45 BF_TEST_GO_AT=4.0 BF_TEST_SPRINT=4.0
  BF_TEST_SKILL=2.0`, and the NPC and the run 0.4 m either side): the body reaches the panel,
  slides down it and lies, no body faster than 3.9 m/s nor turning faster than 7.7 rad/s after
  +0.6 s in any of the three. The first round's build popped there (a calf to 22 m/s, a thigh to
  35 rad/s, the trunk thrown up 0.3 m: 19 frames); with its joint solve put back (the turn rebuilt
  from the clamped angles, about the child bone's axes) the pops came back in all three throws
  (2-10 frames, up to 44 m/s), and with either the rebuild or the child's axes alone they didn't.
  On the open test floor the body leaves at 13 m/s and goes 11 m.
* **A blast on a corpse** (a Frag landing 2 m from Tex's body, `BF_TEST_GRENADE_TYPE=Frag
  BF_TEST_THROW=4.0,0.2` with the NPC at 0.15, -9; #259): "ragdoll blast at 1.97 m of 8.0: 88.7 N s on
  the root"; before #307 the body, lying flat, jolted a few centimetres (friction 4 held it) and
  the loose guns flew; now the damage handler's request throws every body (the S2 row above:
  16 m/s at the jump, 4.8 m), the root's own push on top.
* **Shots into a corpse** (the frozen Tex above with the shooter at 0.4, 8: `BF_CAMERA_PITCH=-0.4
  BF_TEST_TRIGGER=1.5-1.7,4.0-4.2,5.0-5.2,6.0-6.2`): three hits "shot tex's body at ...", each
  bleeding where it struck and jolting the body.
* **The Energy grenade** (grenades.md's command without `BF_TEST_TOGGLE_KILL`): the first blast
  kills Flint and Hawk (both moving: their bodies start at 1.8 x their velocity), the second's
  bolts throw both corpses ("body thrown": Hawk's at 2.4 m/s, 1.7 up, landing 0.6 m off).
* **The knock-down** (`--test BF_MAP=flat BF_CHARACTER=0 BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,8,0,8
  BF_TEST_KNOCK=1.5`): Brutus is thrown back, down within 0.6 s and back up by 1.4 s.
* **The player's death, the death camera and the hand-over** (`BF_MAP=sdm_e34
  BF_TEST_GOTO=15.3,37,15.0,29 BF_TEST_DIE=1`): the guns drop at once, the body holds, falls
  against the rock and 2 m down the slope; the death camera circles it, the memory chip lies
  beside it, "BRUTUS" shows at 4.5 s and control passes at 5.0 s, as squad.md has them.
* **Other skeletons**: e34's level creatures that the squad killed (25 bones, a Spine3) built 19
  bodies in 13 pieces and fell like the rest.
* **Through a hinter** (#298, after main's 9226a99 was merged in): Tex killed on sdm_e13's outer
  lava shore, 2.4 m up the slope from the hinter band (`--test BF_MAP=sdm_e13 BF_NO_SQUAD=1
  BF_TEST_GOTO=98.53,-92.63,115.84,-104.52 BF_TEST_DIE=1.6`) rolls down the shore through the
  band: her root at (103.74, -96.26) at +4.5 s and on into the lava. With her corpse's body made
  a character's (a test edit), the same fall stops against the band at (101.68, -94.56), where
  the #298 run's Roller was pushed out by the old rule.
* **Died knocked down** (`--test BF_MAP=flat BF_TEST_GOTO=0,8,0,8 BF_TEST_KNOCK=1.5
  BF_TEST_DIE=1.55`): "ragdoll tex: died knocked down, its body a thing's now", the knock's
  ragdoll going on as the corpse's, no new build. Before (main's rule too) the hit's `hurt`
  dropped the knock, so the corpse's ragdoll was built anew from the pose before the knock.
* `cargo test --bin bf_play ragdoll` (18): the joint table's names, the angles' round trip, a body
  resting, sliding and spinning to a stop, a bar resting at its stop (frozen within 1.5 s), a body
  sliding along two stops (its energy falling every step, then frozen), the pendulum's period,
  rocking dying down, a chain settling, the movement check's three rules (a still body frozen at
  the second check, 0.52 s; one creeping 6 mm a check frozen by the long rule after 4 s; one
  going 2 cm a check never); #307's: FUN_000e00b0's merging (a kill's two requests stay two, the
  same body's add and blend, another body's replaces only at five times both or more), FUN_00127c60's
  shares (the up-kick's 18 L, a rifle kill's 75 and 300, a blast's factor 0 made 3, a factor under
  1 pushing the one struck alone), its gate (10 m/s passes, 10.1 doesn't: the speed squared),
  and the walk (nothing in the first update, one request an update, a refused one dropped, a factor
  of 0 letting the walk on); #366's: a kill whose record names no body merges with the up-kick
  into one body-less request (take01's rack kill: 16.7, 37.75, the way (-0.027, 0.360, 0.890)), which
  throws every body alike at the cap, a plate's zero way going straight up, and a vaporised body asks for nothing.
  `cargo test --lib arena` (`ragdoll_faces_leave_blockers_to_characters`):
  a blocker's and a hinter's walls are among a knocked-down character's ragdoll faces and not a
  corpse's; an object's box is among both.

## Not done, fits and guesses

* **What the live read settled (#307).** The game does push a dying body: the CBT_DEAD handler's
  two requests land two frames after the build and throw it at ~16 m/s along the shot (a weak hit
  proportionally less), so the demo's old collapse in place, 0.8-1.0 s from its first motion to the
  floor, was the missing push, not a solver's slowness; and the takes' 0.35-0.45 s stand was their
  breakpoints'. **What #366's reads settled:** which kills are thrown (the table under "What the
  game does"): a scenery area's (type 10: body-less, at the cap) and a damage plate's (straight
  up) are, an Energy kill is as a Frag's, a Gas kill, a fall and a liquid's are not (the gate), a
  type-3 kill is vaporised. Not done: the type-3 kill's own look (the bio effect, the carrier's
  chain: #147), the area's range test (#85's fit, `CENTRE_UP` and the archetype's centre, kept: the
  game measures from the object's origin to the hips), the plate's, the fall's and the liquid's
  records (read from the code, not filmed), the bare deal's and a destroyer's kills (no demo path
  but the Vengar's charge, whose own request carries the direction), the Energy's bone body (the
  root stands in), and the Energy's damage coming with the blast and not the bolts (#74's inference).
* **Not matched**: the feet's catch on e34's rising sand (R1, R2: a third too far); the rest after
  a throw comes 0.2-0.5 s early; a lying forearm or head creeps for seconds (a few cm in 8 s, the
  freeze of 4-10 s where the takes' bodies read still states from ~3 s); the world's load L is
  a constant where the game's moves with the live objects (L 0.86-0.97 in the takes: the kick's
  mean speed is 5-10% off by it); a blast's struck body is the root where the game's is whichever
  hit part its ball met first (the spin of a Frag kill: the takes' 45-57 rad/s, the demo's 41 at
  most); a bullet kill's point is the cylinder's, not the part's surface; the shots through a
  corpse still jolt it (`Ragdoll::shot`, 3 m/s: take10's two shells moved nothing and asked for
  nothing, but whether they struck it isn't known).
* Fits and readings: `SHOVE_LOAD` (0.9);
  `FIRST_WALK` (the first shove two updates after the build: the takes' samples, Tex's a frame
  later); the twist-and-swing split of a joint's turn; the knock-down's start; friction 4
  everywhere (not 6 on the "3" grounds: e34's only material 20 is one); the movement check's
  radius (core+4, read as the body's reach) and its fixed 17 steps; `MAX_STOP_TURN`.
* The dropped guns and the loose objects don't touch the ragdoll, nor the living; the bodies of
  one ragdoll pass through each other.
* A running character's bodies start with the 9 x velocity impulse alone (the game's also keep
  the speeds their animation's motion controllers gave them). The running death's 3.2 m slide
  has no take to compare with (#280's shot list).
* Not compared: a body on a slope against a take, the FF2 deaths frame by frame, a death in a
  liquid.
* The bodies' removal (#151).
