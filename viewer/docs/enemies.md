# Enemies: the species' own behaviour (`src/bin/play_enemies.rs`, #129)

What a character of a type other than the squad's does by its own type's data: a test map NPC
of slot `OTHER` ([character-types.md](character-types.md), "In play"). The Shrike Soldier and
Shrike Heavy are the first species (#129); the machinery runs for every such type, the other
species' tickets (#130-#134) check theirs. The squad keeps its own rules: root-motion
locomotion ([play.md](play.md)), its ragdoll knock-down and its decals ([squad.md](squad.md)).

Sources: the spec, footage and the analyst's answers in `todo/129-shrikes/` (`spec.md`,
`notes.md`, `answers.md`), the leads in `todo/110-enemy-species/` and `todo/128-enemy-types/`,
and the default.xbe functions in [code-map.md](code-map.md).

## Spawning one, and its gun

* `BF_TEST_NPC` and the N menu spawn any loaded type ([testtools.md](testtools.md)). A type
  holds its own default gun: the campaign Shrike Soldier the LZR-23, the Heavy the Rail CLVR,
  the deathmatch ones the MK-ASLT and the RVG50 Minigun.
* **A placement** spawns its type with the placement's own weapons and personality:
  `BF_TEST_NPC`'s character may be a placement of a loaded level by its name
  (`<character-object name>`, `h_xxxxxxxx`). The placements of the takes (`--test BF_MAP=e34`;
  their weapons were read live in the takes, spec "Read first"):

  | placement | type | weapon | personality | heavy hit draws |
  |---|---|---|---|---|
  | h_1c5780f8 | Heavy (125 HP) | A50 Bioreactive | h_eed3a76a | KNOCKDOWN 1 : STUN 1 : NOOP 3 |
  | h_16c345fc | Heavy | A50 Bioreactive | h_0bb586cf | KNOCKDOWN only |
  | h_1550b06e | Heavy | Saryl-99 | h_eed3a76a | 1 : 1 : 3 |
  | h_ea37e0b2 | Soldier (62 HP) | Saryl-45 | h_eed3a76a | 1 : 1 : 3 |
  | h_e0a325b6 | Soldier | Saryl-45 | h_0bb586cf | KNOCKDOWN only |
  | h_e9a41520 | Soldier | Rail RVR | h_032b0602 | 1 : 1 : 3 |

  Enemies hold one gun, in the right hand, and nothing on the back (the placements' second slot
  is empty on every enemy; takes 26-33). Spawning the level's own enemies from their triggers is
  #102.
* `frozen` (BF_TEST_NPC's behaviour, beside `dummy` and `fight`) stands one with its brain off,
  as the takes' trainer freeze (brain mode 6): no hit reactions.

## Its motions: the MOTION table

* **Clips by motion type.** The type's motion set's MOTION table, each motion type's clips by
  FUN_00130c50's rule (`Motion::clip`: `Sc_w1_` + the script, or with the gun up the
  ready-script). Its gait's clips come from the table too: the idle is the sequence's default
  motion (idle_stop: what a spawned enemy stands in; take 01's frozen Heavy at 4 m stands in its
  pose, the gun low across the body) and rp_idle with the gun up; walk / rp_walk; the run (the
  Heavy's MT_RUN is rp_run both ways: it has no relaxed run); the dash; the back steps; the
  walking side steps. `BF_ENEMY_LOG=1` prints the table at spawn, each clip with its length, its
  own speed and its root's travel:

  | MT | Shrike Soldier (lf) | Shrike Heavy (hf) |
  |---|---|---|
  | 19 IDLE | idle_stop 0.20 s / rp_idle 2.67 s (and idle_cautious 9.23, idle_general 20.0) | idle_stop 0.20 / rp_idle 2.67 (idle_cautious 4.07, idle_general 16.67) |
  | 47 WALK | walk / rp_walk 1.13 s, 2.52 m | walk / rp_walk 1.33 s, 1.96 m |
  | 34 RUN | run / rp_run 0.73 s, 3.60 m | rp_run 1.00 s, 3.61 m |
  | 26 / 30 MOVE_LEFT / RIGHT | f_side_run_l / r 0.73 s | rp_f_side_run_l / r 1.00 s |
  | 27 / 31 MOVE_*_BACK | b_side_run_l / r 0.80 s | none |
  | 29 / 33, 28 / 32 MOVE_*_SLOW | none | f_side_walk, b_side_walk 1.33 s |
  | 3 BACKPEDAL / 4 BACKSTEP | back_run 0.80 s / none | none / back_walk 1.33 s |
  | 8 DASH | dash 0.60 s | dash 0.83 s |
  | 14 DIZZY | stunned 1.20 s, 2.15 m back / stunned2 1.33 s, 3.96 m | stunned 1.73 s, 2.10 m / stunned2 2.17 s, 1.96 m |
  | 25 KNOCKDOWN | knockdown 2.30 s, 2.84 m to its right / knockdown2 2.00 s, 3.72 m back | knockdown 2.00 s, 0.58 m / knockdown2 2.17 s, 3.52 m back-left |

  The side moves' roots run forward: the body faces the way it moves and the clip twists the
  upper body (the takes: "strafing swings the body 60-90 deg toward the move direction").
* **Moving by its type's speeds** (FUN_0012c6c0), not by root motion: the walking motion types
  (FUN_00126b40: BACKSTEP, the MOVE_*_SLOW, WALK) at `speeds[1]`, the running ones
  (FUN_00126a40: BACKPEDAL, MOVE_LEFT / _BACK / RIGHT / _BACK, RUN) at `speeds[2]`, DASH at
  `speeds[3]`, CROUCH_WALK at `speeds[0]`; the ground velocity eases toward it at 4.5 a second,
  7 slowing down (0x3c05b8, 0x3c05bc), so it slides to a stop when it lets go; a walking clip
  plays at the walk speed over its own (`AnimDef::move_speed`), at most x1.25 (0x3c05cc). The
  reactions, dodges, dives and surprise move by their clip's root (FUN_0012cb40's list).
* **Who moves them.** The demo's enemies stand where they're put; the fighting AI that moves them
  in the game (cover, strafing, attacking) is out of scope. `BF_TEST_NPC_MOTION` drives one in a
  motion type to a point (below), to show the clips and speeds.

## Hit reactions

The game's rule, as the analyst read the brain (`answers.md` Q1, Q2):

* **Heavy or light.** A non-lethal hit that reaches the hit points (after the stamina shield and
  the type's factor) goes on the brain's damage record, which keeps the last 2.7 s (brain+0x238,
  written by FUN_000163f0; FUN_0003dab0, FUN_0003da40). A brain that knows an enemy posts
  EVT_DAMAGE_HEAVY when the record's total is over half the max hit points (0x3a4f74; the record
  emptied), else EVT_DAMAGE_LIGHT (0x3e95f-0x3e9c1). A lethal hit posts neither.
* **The draw.** The NPC's personality's entry for the event (bf/personality.rs: the placement's
  personality, else its type's; a personality's own entry over its parent's): the weights summed
  and a goal drawn (FUN_00054c40). `default` (every campaign Shrike type's, through h_f0330ad7)
  draws KNOCKDOWN 1 : STUN 1 : NOOP 3 on a heavy hit; its light entry (priority -20.2, "TESTING")
  never acts in a fight: EVT_ENEMY_SEEN sits in the brain's queue at 18. h_0bb586cf draws
  KNOCKDOWN only on a heavy hit, and STUN 1 : NOOP 5 on a light one (20.8).
* **An NPC that knows no enemy** (the demo's dummies: nobody in its sight to fight) posts
  EVT_DAMAGED_NO_ENEMIES instead, whose entry posts EVT_USER2, "Involuntary response":
  KNOCKDOWN 1 : STUN 1 at priority 100. So a dummy reacts to every non-lethal hit, within the
  cooldowns. An NPC set to fight, with a hostile in sight, takes the heavy and light path, as the
  takes' alerted Shrikes.
* **When.** The brain takes the event about 0.3 s after the hit (a fit, below) and draws its
  goal then, with the cooldowns as they are then: the goals' starts test them.
* **Cooldowns, and a type without the motion.** A knock-down's start (goal 65, 0x40870) sets the
  knock-down's cooldown to 6 s and the stun's to 3 s; a stun's (goal 63, 0x3f460) the stun's to
  4 s and the knock-down's to 2 s (FUN_000124f0 overwrites them). A goal refuses to start in its
  own cooldown, or when the type has no MOTION of its motion type (MT_KNOCKDOWN, MT_DIZZY: the
  deathmatch Shadoon has a DIZZY and no KNOCKDOWN); the draw strikes it out and draws again
  among the rest (FUN_00054bc0).
* **One at a time.** The reactions are uninterruptible goals (class 0x30 in the goal table
  0x4a5908): a damage event during one waits in the brain's queue (it doesn't decay) and is taken
  when the reaction ends, where the cooldowns mostly refuse it.
* **The motion.** GOAL_STUN plays MT_DIZZY, GOAL_KNOCKDOWN MT_KNOCKDOWN: one of the two clips at
  random (FUN_00150670), once, at rate 1, moving by its root; the state lasts the clip, then the
  idle. The takes' 3.2-3.4 s stuns and 2.6-3.0 s knock-downs were the hit log's stalls: the game
  steps at most 0.1 s a frame, re-timed they come out at the clips' 1.7-2.4 s (answers.md). The
  knock-down is a clip, not a ragdoll: thrown back, it lands, lies and gets up (take 14).
* **Fits and guesses.** The brain takes the event 0.3 s after the hit (a fit: the takes'
  +0.37-0.46 s to the motion state in their column's time, which ran ahead of the game's clock;
  answers.md: about 0.3 s). The body is
  turned to face the shooter first, so the clip throws it away from the hit: the game turns the
  clip by char+0x434 + pi (0x150845), and that char+0x434 is the hit's way is the analyst's guess.
  The MOTION's blend-in (0.2 s hf, 0.25 s lf) is the demo's 0.2 s crossfade.

## Death

* **Limp.** The Shrikes have no death clip (enable-ragdoll-death true, no MT_DIE): the ragdoll
  ([play.md](play.md), the death paragraph), with the game's bodies and joint limits
  (`Ragdoll::new`'s `parts`, `enemies::ragdoll_parts`; answers.md Q6):
  * **Bodies** (FUN_00127290): the game's ragdoll has a body for each of the character's hit
    parts (its type's eleven `<collision>` bones: Bip01, Spine2, Head, the thighs, calves, upper
    arms and forearms) and each of their ancestors up to the root. A bone below the lowest hit
    part of its chain has none and stays as posed at death, rigid with the bone above: a
    Shrike's HorseLink, foot and toes under the calf, its hands under the forearms. So the
    digitigrade lower leg (Thigh -> Calf -> HorseLink -> Foot) keeps its bend from death, and
    only the hip and the knee move. In the demo's point bodies: those bones are held to the bone
    above, to the child it points at and to the one of them farthest off that line, at full
    stiffness, and drawn turning with it.
  * **Limits** (FUN_00113120, by the child bone's name, hard-coded, the same for every
    character): the knee (Calf) is a one-way hinge, 0-90 degrees (its y range, -90..0; x and z
    none), bent the way it was at death; the elbow (Forearm) likewise, 0-100 degrees (y,
    -100..0); the hip (Thigh) swings at most 110 degrees forward, 60 back (y, -60..110) and 20
    to either side (z, -20..20) of its way in the bind pose (the Heavy's thighs: 7 degrees
    forward, 5 out). Not traced, so the demo's reading: which bend axis is which and their
    signs (the joints' frames come from the parent node, FUN_00039510): the larger range is
    taken as the forward swing, as a hip's is; a knee's or elbow's zero is straight, the hip's
    the bind pose. A joint folded past its limit at death keeps that fold (not snapped open).
    The shoulders', spine's, neck's and head's limits aren't applied (the demo's torso pieces
    and head range stand in).
  * **Not modelled:** the hip's twist (x, -45..45: the point bodies have no twist), and the
    knee's and elbow's lock on their lower part's roll about its own line (a lock on the knee's
    plane fought the floor and kept the body writhing, so the lower leg may roll about its line);
    the game's masses (the root 9, the rest 4.5), damping (0.5 / 0.95) and start velocity (the
    character's): the demo's point bodies fall as the squad's.
  * The squad's ragdoll stays the demo's own (every bone moves, knees fold to ~120 degrees, its
    hinges' limits the demo's), though the game's rule is the same for them: a follow-up.
* **The death line.** The type's chatter library (`chatter-library`: shrike_light_chatter,
  shrike_heavy_chatter; dm_shrike_*_chatter in the deathmatch types), its tag e1213623, one of
  its three lines: the Heavy's h_04d16d90, h_00c08720, h_f6aa73af, the Soldier's h_0384c97d,
  h_fb465ea7, h_ecef9910 (the takes heard h_04d16d90, h_f6aa73af and h_0384c97d). It starts in
  the death frame (FUN_00115d50 -> FUN_00125f40 at 0x116028, through the chatter gate
  FUN_00140a60; no delay in the code, answers.md Q8: the takes' 0.15-0.35 s was the capture's
  audio latency). The pick is the game's (FUN_00136fc0 -> FUN_001371f0): at random among the
  lines not said yet on the map (the chatter manager's history, one per world, cleared once all
  are said), so a type's three all come before one repeats (`enemies::death_cry`, `SAID`; the
  skill lines' `LineHistory` keeps the same rule). An enemy says nothing else: the takes'
  Shrikes cried only at the kill, though their files have hurt lines.
* **The deathmatch types' lines** (the dm Heavy's h_0c60112d, h_f6aa73af, h_1bc9d69a) are in
  mp_common's sound banks (`sounds-dm`, `sounds-sdm_shrike`), streamed from their own language
  wave banks (ml-sounds/en/dm-en.tgz, sdm_shrike-en.tgz): the test map attaches them
  (`Game::load_bank_streams`), so they speak there, and on any map played with `--test`. The
  dm h_f6aa73af decodes to the campaign Heavy's samples exactly. The takes' dm kills (36, 39) had
  no cry checked.
* **The gun** leaves the hand and drops at the feet (the takes' guns lie at the standing spot;
  the squad's are thrown by the hit). The placement's `inventory-drop` isn't dropped: the takes
  didn't show one (h_1c5780f8's Sonic Ammo; a shot for it is in the spec's shot list).
* The kill reward text ("+     300") is #176.

## Blood

* **Shrike flesh** (world material 23, FLESH_SHRIKE: read from the parts' collision) has its own
  debris bundle, h_14559f7c: blood_puff#1, bloodsplat_s#1 and giblet#1 (common.ale), green. Its
  hit effects and sounds per ammo type are FLESH_HUMAN's (play_fx.rs `BLOOD_1`, `SHRIKE`).
* **The game's decal rule** (FUN_00124a70), for Shrike flesh only (the other characters keep the
  demo's reading until #150). a = the amount over the type's damage-multiplier:
  * the first list (h_fb4bfc68; the Shrikes' h_ecfc752b or h_f7f52491 at random, FUN_00124af0):
    when a is at least 5% of the max hit points or at least the hit points left; on the floor at
    the middle of the body and the hit point, moved up to 0.3 m at random, turned at random;
    scale min(1 + 0.05a, 2), doubled when a is more than the hit points left (every kill, and on
    the 62-HP Soldier any hit over 31); alpha 0.5 + 0.5 min(0.05a, 1);
  * the second list (h_efc2a874; h_1c2da148, FUN_00124f60): while it lives and a is at least 10%
    of the max: on the first surface within 4 m behind the hit along the shot (taken level),
    facing back; scale min(0.5 + 0.05a, 1), alpha 0.8 + 0.2 min(0.05a, 1). Nothing past 4 m;
  * no death splats and no pool: the killing hit's own decal (doubled) is all.
* **How they're drawn** (answers.md Q4): multiplied into what's under them, as the game's decal
  pass (FUN_002175e0: SRCBLEND ZERO, DESTBLEND SRCCOLOR; pixel shader 25): what's under it times
  lerp(1, min(1, tint + 1 - the texture's alpha), a), a = the tint's alpha (220) x the decal's
  alpha x its fade; the texture's colour isn't used. The pass's shader lerps that factor to 1 by
  the fog, so in the fog a decal fades to no effect: Bevy's fog is off on these decals (fogged,
  the factor's white took the fog's colour and the whole quad showed, ~10% darker than the floor
  at 60 m on e34), and their alpha is scaled by 1 - the fog at the decal each frame (`animate_fx`,
  `fog_at`). Widths and heights are half sizes (a kill's
  0.5 x 0.25 decal at scale 4 is a 4 x 2 m quad; its visible patch is ~1.5 m, the texture's
  alpha). Take 20's kill spot: a floor of 35, 28, 36 darkened to 15, 23, 17 (the formula gives
  19, 22, 20).

## The stamina shield (the deathmatch Shrikes)

* The types whose shield-type is 2 carry a stamina shield: the deathmatch Shrike Soldier, Shrike
  Heavy and Hunter Lord, and their squad deathmatch copies (the rule goes by the shield type). A
  hit takes the stamina first, min(stamina, damage) off both (0x223690), and what's left goes to
  the hit points in the same hit. Their stamina is the type's max-stamina: 25, 40 and 35 (dm), 28,
  45 and 40 (sdm). A hit the shield takes whole draws no blood and lays no decal. The test map's
  instant kill counts the stamina in (`enemies::lethal`), so K kills them in one hit.
* It refills at the type's stamina-recovery a second, 5 (Soldier) and 4 (Heavy), x0.25 dashing
  and x2 crouched, with no pause (FUN_0011a840; answers.md Q5). The takes' steps of recovery / 10
  were frames whose time the game capped at 0.1 s while the hit log paused the console.
* A hit the shield takes plays the shield's hit-effect h_1218cdc1 (the ALE effect `pshield`)
  round the body. It doesn't show: `pshield`'s emitter nodes are of classes bf_viewer's ALE
  player doesn't run (the takes' blue-white shimmer round the victim).

## Test hooks

* `BF_TEST_NPC=<s>,<character>,<team>,<dummy|fight|frozen>[,<x>,<z>[,<y>]]`: the character may
  be a placement name (above); `frozen` has no reactions ([testtools.md](testtools.md)).
* `BF_TEST_NPC_HP` (testtools.md) sets the maximum hit points too, and the heavy-hit threshold
  and both decal thresholds are shares of the maximum: at 999 a Shrike takes no heavy hit and
  lays almost no decals.
* `BF_TEST_NPC_MOTION=<s>,<n|aim|all>,<motion>[,<x>,<z>[,relaxed]]`: drives an enemy NPC in a
  motion type (its MT_ name or number: `walk`, `run`, `dash`, `move_left`,
  `move_right_back_slow`, `backpedal`, `backstep`, `idle`, `crouch`, `dizzy`, `knockdown`,
  `dodge_left` ...) to x, z, or in place without them. Its clip is the MOTION table's
  ready-script's (the gun raised, as the takes' alerted Shrikes; `relaxed`: the script's); it
  moves at its type's speed for the motion type, the body turned at the type's turn-rate to where
  the clip carries it; at the goal it lets go and slides to a stop. A motion that plays once (a
  reaction, a dodge) plays through in place. A hit cuts the drive short.
* `BF_ENEMY_LOG=1`: an enemy's MOTION table at spawn, each hit's damage record and event, the
  personality's draw and refusals, the motions as they start and end, the drive every half
  second (position, speed, facing), the stamina shield, the death line picked (and how many of
  the set weren't said yet).
* `BF_RAGDOLL_LOG=1` also prints, as an enemy's ragdoll is built, how many bones have bodies,
  the bones rigid below each, its knees' and elbows' bends at death and the thighs' bind-pose
  ways the hip limits are about.
* `BF_COMBAT_LOG=1` also prints each game-rule decal (its list, place, scale and alpha) and the
  stamina shield's share.

## Verified

Captures at 15 fps with `BF_MUTE=1 BF_BACKGROUND=1`, frames looked at against the takes in
`todo/129-shrikes/`. "The range spot": `--test BF_MAP=e34 BF_CHARACTER=1
BF_TEST_GOTO=-53.6,30.8,-53.71,31.49 BF_TEST_FIRE=hip BF_CAMERA_PITCH=-0.12`, Tex firing her
L-Shot at an NPC 5 m off, at the takes' place and light.

* **Placements and guns** (`--test BF_MAP=e34
  BF_TEST_NPC="0.3,h_1c5780f8,enemy,dummy,-54.6,34.8;0.3,h_e9a41520,enemy,dummy,-52.6,34.8"
  BF_ENEMY_LOG=1`): the Heavy holds the A50 and the Soldier the Rail RVR, one gun each in the
  right hand, nothing on the back (takes 26-33); their personalities h_eed3a76a and h_032b0602.
* **Idle** (`BF_TEST_GOTO=60,-9,59.955,-8.30 BF_TEST_NPC=0.3,h_1c5780f8,enemy,frozen,58.93,-5.15
  BF_TEST_NPC_MOTION=0.4,1,idle` on e34): the frozen Heavy stands in idle_stop's pose as take
  01's at 4 m, the gun low across the body, the feet on the floor.
* **Walk, run, strafe, backpedal, dash** (`--test BF_TEST_GOTO=0,8,0,8
  BF_TEST_NPC=0.3,shrike soldier@dm,enemy,dummy,-6,-4
  BF_TEST_NPC_MOTION="1,1,walk,6,-4;5,1,run,-6,-10;7.5,1,move_left,6,-10;10,1,backpedal,-6,-10;12.5,1,dash,6,-4"
  BF_TEST_FREECAM=0.5,0,9,10,0,-40 BF_ENEMY_LOG=1`): the drive log's speeds level at the dm
  Soldier's 2.50 (walk), 6.50 (run, side run, backpedal) and 8.50 m/s (dash), its `speeds`, the
  takes 34 / 37's tops; each with the table's clip (the side run facing its way, the upper body
  twisted), easing in and sliding to a stop.
* **Reactions** (the range spot, `BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC=0.3,h_16c345fc,enemy,dummy,-54.5,35.8 BF_COMBAT_LOG=1 BF_ENEMY_LOG=1`, 150
  frames): the dummy knows no enemy, so every hit is EVT_DAMAGED_NO_ENEMIES -> EVT_USER2. The
  first hit (0.87 s) is taken at 1.20 s and draws a knock-down (knockdown, 2.00 s): thrown back,
  it lands, lies and gets up, as take 14's; the second, during it, waits and is refused by both
  cooldowns when it ends (knock-down 4.0 s left, stun 1.0 s); the third (4.27 s) is taken at
  4.60 s, where the knock-down's cooldown refuses it and a stun is drawn (stunned, 1.73 s, then
  the idle).
* **A type without the motion** (`--test BF_CHARACTER=1 BF_TEST_INSTANT_KILL=0
  BF_TEST_GOTO=0,8,0,7.3 BF_TEST_NPC=0.3,h_f1dead6f,enemy,dummy,0.15,3 BF_TEST_FIRE=hip
  BF_ENEMY_LOG=1`, the dm Shadoon): its draw struck KNOCKDOWN out ("refused [65]") and played
  its stun.
* **Death** (the range spot, `BF_TEST_NPC=0.3,h_1c5780f8,enemy,frozen,-54.5,35.8
  BF_TEST_FREECAM=1.1,-49.5,31.0,36.4,90,-14 BF_NO_HUD=1 BF_RAGDOLL_LOG=1 BF_SOUND_LOG=1`, 90
  frames): at the kill the green mist, the gun drops at the feet; the body goes limp, thrown back
  1.4 m, the legs buckle and it lies folded on its side ~1.2 s after the hit (take 21: 1.1-1.3
  s), at rest ~2 s after. Its lower legs keep their bend from death, the feet raised off the
  floor; "ragdoll: 45 bones, 18 with bodies; rigid below: ... Bip01 L Calf +3, ... Bip01 L
  Forearm +10 ...; bent at death: Bip01 L Calf 54 deg ...". The death line
  h_04d16d90 plays on the frame of the killing hit's sound. The takes' resting bodies are too far
  and dark to compare the legs' pose closely (takes 19 and 21: a low folded heap).
* **Death lines** (the same, four Heavies and then a Soldier spawned in turn at the same spot,
  `BF_TEST_NPC="0.3,h_1c5780f8,...;2.5,...;4.7,...;6.9,...;9.1,h_e9a41520,..." BF_ENEMY_LOG=1
  BF_SOUND_LOG=1`): the Heavies say h_04d16d90, h_f6aa73af, h_00c08720, then one of all three
  again ("3 not said yet"); the Soldier one of its own three (h_ecef9910). Each starts on the
  frame of the killing hit's sound. On the test map the dm Heavy (`shrike heavy@dm`) says its
  own (h_0c60112d), from mp_common's dm bank.
* **Decals** (the range spot, `BF_TEST_INSTANT_KILL=0 BF_TEST_NPC=0.3,h_1c5780f8,enemy,frozen,-54.5,35.8
  BF_TEST_FREECAM=9.0,-54.5,33.6,34.6,180,-72 BF_NO_HUD=1 BF_COMBAT_LOG=1`): each hit's list-1
  decal on the floor between the feet and the hit, scale 2, then 4 once the hit is more than
  the hit points left (as take 17's), alpha 1; the second list only within 4 m behind: in the
  reactions run above, the hits after the stun had carried the Heavy 2 m back laid theirs on the
  gate panel 3.7-3.8 m behind, at z 41.32 facing back, as take 14's; the patches darken the
  floor dark green, multiplied, as take 20's kill spot.
* **Decals in the fog** (the range spot with the kill, `BF_TEST_FREECAM=2.5,-54.5,79.2,-14.6,180,-45`:
  the kill decal from 70 m above and behind, and from 90 m, `...,93.0,-28.4,...`): before the fog
  fix the whole 4 x 2 m quad showed as a pale green parallelogram round the patch; now only the
  patch shows, faint at 90 m, and from 15 m it looks as before.
* **The stamina shield** (`--test BF_CHARACTER=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=0,8,0,7.3
  BF_TEST_NPC=0.3,shrike heavy@dm,enemy,frozen,0.15,3 BF_TEST_FIRE=hip BF_CAMERA_PITCH=-0.12
  BF_ENEMY_LOG=1`): the dm Heavy's 40 stamina takes the first L-Shot hit's 37.4 whole (no blood,
  no decal), refills at 4 a second, and the next hits take what's left from the hit points. With
  the instant kill on, one hit of 95 (its 55 HP and 40 stamina) kills it.
* **No regressions** (main's build, 0681343, against this one, frame by frame): the autopilot on
  sdm_e34 (293 of 300 frames the same: the first frames, which differ between two runs of one
  build too) and `BF_TEST_KILL=3` on the test map (118 of 120, a few pixels; the same combat log
  and the same sounds, so the test map's new streamed banks change none of the squad's lines).
  Before the merge, against 8b5f701: the NPC fight of testtools.md's "Verified" (202 of 225, the
  same combat log) and the player's death on sdm_e34 (91 of 100, the loading frames). The squad's
  ragdoll, decals, knock-down and chatter are untouched.

## Not done

* The fighting AI: moving, cover, strafing, attacking; when an enemy walks, runs or strafes. The
  hook shows the motions it would play.
* Spawning the level's enemies from their spawn triggers (#102).
* The glow of their skins and their own capsules (#173): the hit cylinder is the squad's, also
  through a knock-down (a body lying on the ground is hit through the standing cylinder).
* The kill reward text (#176); the placements' `inventory-drop`.
* The other characters' decals by the game's rule and blend (#150).
* The stamina shield's shimmer (`pshield`: its emitters aren't run by the ALE player).
* Footsteps of a driven enemy; its idle variations (the unalerted enemy's idle_general and
  idle_cautious, the AI's); its hurt lines.
* Where the corpse rests: the takes' body-shot Heavies lay ~1.5-2 m behind their standing spot,
  the head-shot one folded down in place; the demo's ragdoll throws them back ~1.4 m. The
  game's masses, damping and the lower leg's roll lock (above) aren't the demo's.
* The squad's ragdoll by the game's rule (its bodies and joint limits, FUN_00127290 /
  FUN_00113120).
* The tester's notes against the takes: the green mist covers about 0.44 of the take's area; the
  Soldier's knock-down slot 1 (knockdown, 2.84 m to its right) throws it sideways; the second
  list's ray is taken level.
