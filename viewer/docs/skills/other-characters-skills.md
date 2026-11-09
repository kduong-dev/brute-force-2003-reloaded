*Part of [skills/](README.md), the former page `skills.md`.*

## Other characters' skills: the effect skills and the Orbital Beam (#200)

Beyond the squad's four, a character type's `<combat-skill skill-set>` picks one of 15 more skill
classes (the factory FUN_0012e590; the analyst's map is `todo/npc-skills/answer.md`). The
**effect skills** are a family: the Orbital Beam (CSKILL 9), the Air Strike (6, #203), the Wrath
of Garner (13, #208), the bosses (17, 18, 21) and the Molotov Mutant (20, #211: [its page](molotov-mutant.md)) each start an effect type
at a point through one spawner (the ferals' Spirit of Vengar and Spirit of Garner, 14 and 15, aren't
of the family, being Brutus's skill over their own blocks, but are handed out through its three
dispatch points: [their page](ferals-spirits.md)). #200 builds that shared part and the Orbital Beam
(`src/bin/play_effect_skills.rs`); #203 adds the Air Strike (`src/bin/play_air_strike.rs`:
[its page](air-strike.md)); the effect is started as any effect type is
([effects.md](../effects.md)). From default.xbe (the rows are in
[code-map/skills-and-stamina.md](../code-map/skills-and-stamina.md): 0x13df70, FUN_0013e410,
FUN_0013e600, FUN_0013e740, FUN_0013de50, FUN_0013e910, the vtable 0x39cae0), the analyst's answers
(`todo/200-effect-skills/answers.md`) and the 18 takes in `todo/200-effect-skills/` (the spec is
`spec.md` there, §1 and §2).

### Who has the Orbital Beam

| Type | Where | HP | Cost / recovery |
|---|---|---|---|
| Confed Marine, h_048b8580 / h_fe5ef6c4 | deathmatch (objecttypes-dm) / the Ferguson Platoon (sdm_ferg) | 80 | 55 / 2.3 |
| Sgt. Gonzales h_ef8787e4, Private Cogburn h_f48ed65e, Corporal Stout h_fd89e6c8 | e01, e05, m01_a, m01_b, m02_a/b/c, the tutorial | 90 | 99 / 0: one beam a life |

None of their types names a start or end sound the class would play: it never switches on. In the
demo a **type of its own** (the test map's squad slots, #302) takes its type's Orbital Beam
(`effect_skills::own_type_skill`, play.rs `spawn_unit`): with the Militia's Air Strike (#203), the ferals' spirits
(#202, #209), the Outcast Shaman's Wrath of Garner (#208: [its own page](wrath-of-garner.md)), McTavish's Tracking (#207: [its own page](mctavishs-tracking.md); Flint's skill over his own block, in the
same three dispatch points), Gunthar Ghent's Cloak (#205: [its own page](ghents-cloak.md); the Cloak starts no effect, it has an arm in the three
dispatch points for the wiring's sake) and, for the demo's
Edward Kingman class, Tex's Berserker (skill-set 3, #407, below) and the Molotov Mutant's Self-Destruct (skill-set 20, #211: [its page](molotov-mutant.md);
the target point there is the character's own root node, `SkillKind::own_point`, not the aim's) the only classes a type of its own
gets; every other keeps "has no special skill". Its stamina is then this page's (the skill's stamina
update), not play_enemies.rs's refill (`skills::refills`), so it refills once. The family's three
dispatch points are in play_effect_skills.rs, one arm a class: `kind` (the skill object by skill-set),
`own_type_skill` (who gets the class from its type) and `give_defaults` (BF_TEST_GIVE_SKILL's cost and recovery).

### The shared part

* **The spawner** (FUN_0013e410): a skill asks for it by setting `Skill::fire` (its toggle, its
  update, an animation event); `effect_skills::fire_effects`, after the skills' frame and before the
  effects start, finds the target point and starts `count` effects of the skill's `<effect-data>`
  (`bf/character.rs` `EffectData`, FUN_0013de50's copy: count, effect, XZ scatter, pitch, re-roll,
  reach, rise), each at the point plus a random offset within +-the scatter on x and z
  (FUN_0013e740), turned by the pitch and a random yaw (FUN_0013e800, FUN_0001fa20: the pitch first, then a swing
  about the vertical, as the Air Strike's takes show), its user
  parameter 1 (1 - a random share with the re-roll), owned by the skill's character
  (`EffectSpawn::of_skill`, FUN_0013e600). Only the record's first effect name is ever copied, so
  one effect type a skill. The Orbital Beam's record: one h_ff8e79d6, no scatter, no pitch, reach
  50, rise 0.
* **The target point** (0x13df70, `effect_skills::target_point`), for the character a player
  controls; written once at the spawn into `Skill::point`:
  * **locked** (the reticle's lock: a hostile in the lock box, or under the centre where the gun
    takes that): the lock target's root node, its hips (`missile::root_point`): Tex's feet + 1.19 m,
    as takes 01, 05 and 08 read it;
  * **no lock**: a ray from the camera's eye (`Reticle::eye`) toward the reticle's centre-ray point
    (`Reticle::point`), 50 m long, against the world (no blockers) and the other characters' bodies.
    The terrain: its point, raised by the rise; an object's face or a body that doesn't look down
    (a wall, a floor mesh, a body's side): its point; a face that looks down, or nothing within 50 m:
    a second ray straight down from 1 m out along the face's normal (from 1 m under the reach's end
    with nothing met), 62.5 m long, against the world alone: where it lands, raised by the rise, or
    50 m under its start. A teammate isn't locked (only a hostile locks), so the ray meets his body.
  * Anyone else: as it was. The AI's goal (GOAL_TOGGLE_COMBAT_SKILL's start, 0x36e80) would write
    its target's position there, refusing without a target or with a friend within 20 m of it
    (FUN_00034b20); the demo's goal does neither, so it refuses an attack skill's order
    (play_orders.rs `toggle_ability`, the skill's +0x10c: `SkillKind::attack_skill`).
* **The damage** is the effect's ([effects.md](../effects.md), `Rules::Game`): its clock from the
  spawn (the first tick in the frame of the press), each record `amount` x dt to everyone whose root
  node is within its range of the point, **the owner and his team included**, at x1.0 (no FRIENDLY
  share: the record isn't a weapon's), each hit the owner's (the record's attacker: the owner's kill
  line, `Cause::by`). The ALE effects start at their alchemy entries' creation-times (the Air Strike's
  shell at 2 s), and the owner's red screen tint needs an area hitting him at more than 30 HP a second (#203, strict
  since #208: [The Air Strike](air-strike.md), [effects.md](../effects.md)).
* **The HUD name**: a skill's own hold of its name (`Skill::name_hold`, the name widget's timer
  `[view+0xfc]+0x74`) shows it at full while it runs, then it fades at 3 a second as after an off
  ([HUD](hud.md)).

### The Orbital Beam (FUN_0013e910)

* **The press** (V, the White button; `BF_TEST_SKILL`): with stamina at least the cost, the effect
  is asked for, the cost is taken at once (100 -> 45) and the name held 0.2 s; under the cost,
  nothing at all (`Toggle::Refused`: no name, no sound, no effect). It answers `Toggle::Stays`: no
  switch, so no on state, no start or end sound, no line, no drain and no motion; the stamina refills
  at its recovery all along (2.3 a second). Free to act always (+0x14 0x213690): nothing it gates.
* **The effect** h_ff8e79d6: its three ALE effects by the game's count and shapes (`exp_ionbeam`'s
  `godray.tga` column is a shape-3 quad standing on its particle, its emitter 55.6 m up, so it reads
  as a column from the sky; `exp_ionbeam_hit`'s glow; `exp_ionbeam_pre`'s flash and shock), its
  sound h_f76d4757 the frame after, and its two areas: 300 LASER a second for 0.1 s within 5 m, then
  15 a second for 1 s within 15 m from 2 s, no falloff, push 2.

### Demo-own skills for the test map's classes (#407)

The product owner's requests, **not the game's** (each class's `why` row says demo-own, guess;
`character_class.rs` `Class::skill`, `SkillOverride`): the **Red Hand** and the **Subversive** (their
campaign types have skill-set 0) carry the Confed Marine's Orbital Beam, skill-set 9, cost 55,
recovery 2.3, max-stamina 100 (`ORBITAL_BEAM_DEMO`: objecttypes-dm.xml's Marine); **Edward Kingman**
carries Tex's Berserker, skill-set 3, cost 6.5, recovery 1.2 and Tex's start and end sounds
(`BERSERKER_DEMO`: the deathmatch Tex's, objecttypes-dm.xml line 51, the product owner's choice), in place of CSKILL_KINGMAN 5. `Class::apply_seated` writes the
values into the type's skill block, so the type is a type of its own with that skill-set and
`own_type_skill` hands it to `Skill::new` as it does the Marine's. The **Terrorist** carries **Gunthar
Ghent's Cloak** (#205), skill-set 8, cost 6, recovery 1.5, max-stamina 100 and no sounds (`CLOAK_DEMO`: the
deathmatch Gunthar Ghent's numbers, objecttypes-dm.xml line 415), the product owner's request, the same kind of
demo-own choice: [his Cloak](ghents-cloak.md). The **Filthy Mutant** and, by the product owner's request of 2026-11-09, the **Ugly Mutant** carry the m03 mutants' **corpse eater** (#210), skill-set 16, cost 3, recovery 1.5, max-stamina 100 and no sounds (`CORPSE_EATER_DEMO`: the m03 Filthy Mutants' numbers, objecttypes-m03_a.xml's h_1662a599), the demo's own player skill: [the feed](corpse-eater.md). No other class has an override.
A Berserker on a type of its own has the berserk motions and toggle overlay clips its model and
motion set have (`ms_tex`'s `Sc_w1_bz_*`, found through the type's motion set, not the squad
character's name, so the deathmatch and sdm_bf Texes reach for the rack and put the gun back:
[Tex's Berserker](texs-berserker.md)). Kingman's model and `ms_kingman` have none: the press tops the
other gun up, draws it and drains the stamina, and the character keeps its own motions. **The second gun and the left arm (demo-own,
the product owner's choices):** only Tex's model has the SECOND_HAND hardpoint h_15ab6d29 and berserk clips.
For a model without one (`play_mirror_arm.rs`, `MirrorArm`): the second gun is held on the **left hand at the
mirror of the trigger hand's mount**, and while it is drawn the character step (play.rs) poses the **left arm as the mirror of
the right** after the aim has turned the right one to the crosshair, so the left gun follows the aim and the
recoil. The bones are the Biped's `Bip01 R|L` `Clavicle`, `UpperArm`, `Forearm` and `Hand` (the names of
play_damage.rs `HIT_BONE_NAMES`; Kingman's skeleton has all eight, both clavicles hanging from one spine bone).
The reflection is across **a plane chosen by what the right arm is doing** (#409, `plane_normal`):
* **Not aiming** (standing, walking, running, the gun hanging at his side: the aim's weight,
  `Player::aim_weight`, is 0): the **torso's own plane**, the one between the shoulders (its normal the line
  from the left upper arm to the right, in the neck, the bone both clavicles hang from, so it turns with the
  shoulders). The arms are plain mirrors of each other: both hang, both swing, both guns are held alike.
* **Aiming or firing** (the aim's weight is 1): the **aim's vertical plane**, the plane through the point
  between the shoulders that holds the right gun's barrel direction and the up axis (round 3 of #407: a
  mirror across the torso's own plane doubled the 16 degrees a standing body stays off the aim, so the
  left barrel splayed 25 - 30 degrees outward and the hand sat farther from the body).
* **Between**: the two normals blended by the aim's weight times how far the barrel's heading can be
  trusted (`MIN_HEADING` 0.2, `FULL_HEADING` 0.4 of the barrel's length horizontal: none within 12
  degrees of the vertical, a barrel 78 degrees or more up or down; all from 24 degrees off it, 66 up or
  down or less, past the aim's 60 degree pitch limit), so the plane never jumps as the gun comes up to the
  aim and goes down from it.

(#407's version took the aim's plane whenever the barrel was more than 12 degrees off the vertical. A
hanging gun's barrel is 73 degrees below the horizontal in Kingman's carry idle (`BF_MIRROR_LOG`:
"barrel yaw -51.2 pitch -73.4"), so its heading, a few centimetres of horizontal run along a barrel that
points down, was anywhere: the plane stood 60 degrees off the torso's, the left hand ended in front of his
crotch with the gun on its side (its muzzle at x -0.08, 0.08 m to his right of the centre line, where a
mirror puts it 0.28 m to his left: +0.29 now) while the right hung at his side (the product owner's
screenshot, #409), and in a walk the left arm wobbled as the heading wandered with the swing.) The left arm is also eased in
over 0.25 s as the second gun is drawn, and out as it is stowed (`Player::mirror`, the blend of
`MirrorArm::apply`): it had snapped. A bone's
frame `W` reflected with the matrix `M` is `M W K`, `K` a constant reflection in the bone's own axes that turns the right
bone into the left at rest (`K = Rr0^T M0 Rl0`, `M0` the bind pose's symmetry plane, from the shoulders in the bind
pose), so each left bone is `M Wr K`: with the barrel in the plane its reflection is itself, so both barrels are
parallel to the aim, the hands are level and the same distance from the aim line, and only a shift along the aim of
the shoulders' offset from the plane remains. The left gun is held at `(K^T r S, K^T t)` for the right mount
`(r, t)`, `S` the reflection of the gun's own lateral axis (the one of its two axes off the barrel that the muzzle
has no offset from the grip along), which keeps its barrel and its up: the left gun has the right one's orientation,
grip down. A model without those bones holds the gun at its SUPPORT_HAND
h_f8474385 (Kingman's compound h_e0b89937 has it, objects-m03_c.xml 9868), and one with none of the three
can't draw a second gun: `skills::other_gun` refuses, the fire step's second gun needs a mount, so there is no
phantom fire. Tex's own berserk is untouched (his model has a SECOND_HAND, so he has no `MirrorArm`).

**Kingman's two holsters, his two hands and the swap (demo-own, the product owner's design: #409, `play_holster.rs`; #410,
`play_reach.rs`).** His compound has no stow hardpoint (its 17 hardpoints are the two hands', the head's and the bones' effect
points: objects-m03_c.xml ~9868), so a gun that wasn't in his hand was hidden. The type that carries the Berserker, on a model
with none of the three stow hardpoints (`weapon::STOW`) or the SECOND_HAND, gets **a holster on the side of each hip**
(`Holsters::new`; every other model, Tex's ramps and the squad's hips included, is as it was), and **each gun belongs to a side
and its hand** (`HeldWeapon::left`): the first slot's on his right hip and in his right hand, the second slot's on his left hip
and in his left hand. The game's two stow places the guns' slots start on (`StowPlaces`: place 0 the first gun's, place 1 the
second's) are the two holsters. (#409 hung both guns on his left thigh, the one not in his right hand there; round 1 of #410
moved that one holster to the front of his waist for a cross draw by the right hand. The product owner asked for one holster on
each side instead, each hand reaching only its own.)
* **The holsters.** The right one is Flint's stow hardpoint, the squad member who wears a pistol on his hip (his place 1,
  h_f4ccf78c on his right thigh, objects-common.xml h_160a1622: point -0.143255 0.12138 0.087056, ORIENTATION 0.491457 0.464461
  0.457451 0.577481), its point scaled by the thighs' lengths (Kingman's 0.511 m to Flint's 0.457) and moved 0.10 m back along
  the thigh frame's z (`HIP_SHIFT`, #409's fit: Flint's holster sits where a hand hangs); then, as his idle clip's first frame
  has the thigh, 0.12 m up the side of his hip (`HIP_LIFT`), onto the bone the thighs hang from (`Bip01 Pelvis`, where a belt
  rides, so it no longer swings with the thigh). The left one is its mirror across the pelvis's own middle plane, the one
  between its two thigh joints in the bind pose (`mirror_on_bone`: the mirror the left hand's mount uses, `K` that plane's
  reflection), so the two hang as mirror images on his body. The pistols' HOLSTER hardpoints h_10ab40ac (the four in
  objects-dm.xml and objects-m03_c.xml, the Kman Auto's among them) are all turned alike, ORIENTATION 0 0 1 0 (the rifles'
  another), so the Kman Auto hangs barrel down as Flint's LZR-10 does, its grip about 0.1 m above his hip joints, 0.2 - 0.28 m
  out to the side and 0.13 m behind them (`BF_REACH_LOG`, standing: his idle leans his hips, so the two sit a little differently
  in the model's space), beside the hand that hangs on that side.
* **A gun in each hand.** The gun from the right hip is held and fired with the right hand, the one from the left hip with the
  left (`build_weapon`: the left gun's mount is the mirror of the trigger hand's on the left hand, `MirrorArm::mount`, as the
  berserk's second gun's was). **With the left one in hand he is left-handed** (`Player::hand`, `MirrorArm::swap`): after the
  clips and the overlays his arms are exchanged, each posed as the mirror of the other across the torso's plane, so the left arm
  carries, swings, reloads and raises the gun to aim as the clips have the right do it, and the right hangs and swings empty as
  they have the left. The aim then turns his spine for the gun in the left hand as it does for the right's (`Loaded::arm_chain`: the
  spine bones both clavicles hang from), so the muzzle, the flash, the light and the shots are the left gun's. The chest keeps the
  clips' own turn for a right-handed shooter (only the arms are exchanged), so to aim the left gun the spine and the feet turn a
  little further right (standing, his heading -20 degrees against -16 with the right gun, the aim's yaw 0). The exchange eases in or
  out over 0.25 s as the swap passes the gun from one hand to the other. (His model has no throw clip, so he throws no grenade and
  holds no medkit in either hand: `throw_hand` is None for him.)
* **The swap** (Q). His motion set has no switch clip, so the switch had been instant. The demo gives him a switch of its own
  with the squad's machinery (`start_switch`, `Switching`; `switch_clips_of`: a clip that animates no bone), and **his hands do it
  side to side**: at the switch's drop (0 s, `SWAP_DROP`) the hand in use takes its gun down to its own hip and lets it go there
  (0.25 s); at the grab (0.25 s, `SWAP_GRAB`) the other hand goes down to its own hip, takes the other gun (0.5 s) and brings it up
  (0.75 s, `SWAP_TIME`); nothing aims or fires until then (the aim comes after, as after any switch). The arms' exchange runs from
  0.25 to 0.5 s, as the empty hand comes back up and the other goes down. No reach crosses his body. (Round 1's right hand carried
  the gun across his front to the one holster and back in 0.55 s.)
* **The berserk draw and stow.** The Berserker on a type of its own has no toggle clip, so the skill draws the second gun the
  moment it comes on (`skills::other_gun`: the slot not in hand); the gun stays on its own hip while **its own hand** goes to it
  (0.25 s), takes it and brings it up into the mirrored pose (0.25 s), and only then does it fire. The stow at the skill's end
  takes it back to its own hip the same way. The mirrored arm (`Player::mirror`, play_mirror_arm.rs) is the one without the gun in
  hand: the left, or with the left gun in hand the right (`MirrorArm::apply` from the left, across the plane of the left gun's
  aim); it is wanted while the second gun's own hand holds it, so it eases in as the hand comes up with it and out as the hand
  comes back empty. A switch can't start while either hand is on its way, and a pickup's clip waits for both hands.
* **The arm** (`reach::pose_arm`): a two-bone IK on `Bip01 R|L UpperArm`, `Forearm` and `Hand` over whatever the clips, the
  exchange, the aim and the mirror made of the pose, so the legs keep walking or running under it. Each hand goes to its hip while
  the gun the game has it hold (the gun in hand if it is that side's and held, or the berserk's second gun if that is) isn't the
  one it holds, takes it or lets it go there, and comes back (`Reach::step_to`; a weapon pickup's clip moves its guns itself, so
  the hand takes them as they come). The gun's grip point follows an eased path (smoothstep, `Reach::at` moving 4 a second,
  `LEG_TIME` 0.25 s) from where the hand holds it to where it is at the holster (`holstered`, `reach::Holstered`), bowed out in
  front of the body by 0.45 of the width it crosses (`BOW`: a few centimetres down his own side); the elbow turns from the pose's
  own bend toward its pole, out and back (`POLE_RIGHT`, `POLE_LEFT`); the hand turns from its own turn under the forearm to the
  turn that holds the holstered gun in its mount, as far along as the leg is, so a gun stays in its mount in the hand all the way
  and at the hip is exactly where the holster has it: nothing pops when it changes hands. At the start of a leg the IK gives the
  pose back exactly, so the arm doesn't pop either. What remains of round 1's cross reach: the clavicle's shrug where the arm alone
  falls short (`CLAVICLE_REACH`) is kept, but no reach to its own hip asks for it (at most 0.1 degrees in every run,
  `BF_REACH_LOG`); the spine's turn toward the far hip, the 0.05 s hold at the hip and the gun's turn in the palm are gone (the
  wrist now turns all the way, at most 60 degrees from straight, the idle's own). A switch cut short (a knock-down) between the
  drop and the grab gives the gun back to the hand (`Player::cancel_switch`): one still on its way down with it turns back with it,
  and one that has just let go at the hip takes it back there on its next step and eases back.

Checked (#407; `--test` with `BF_CHARACTER=brutus|hawk|tex BF_TEST_SQUAD_PRESET="0.3,terrorist_cell"
BF_TEST_SKILL=...`, `BF_SKILL_LOG=1`, `BF_CAPTURE` frames): the Red Hand and the Subversive press V at
2.5 s: "orbital beam: stamina -55 -> 45.00", "Orbital Beam" on the HUD, the beam and its flash land, and
the press at 5.0 s with 50.8 is refused under 55; stamina refills at 2.3 a second. Kingman: the weapon
box lists two Kman Autos, the weapon switch swaps them, each fires and a reload takes 9 rounds from the one
reserve (600 -> 591); the press draws the second as the second gun, the HUD says "Berserker", the stamina
drains 6.5 a second and both guns show in the box. In berserk (`BF_TEST_FREECAM` views, 15 fps frames: from
above, both arms raised forward, a Kman Auto in each hand and a muzzle flash at each; from his side with
`BF_CAMERA_PITCH` 0, 0.5 and -0.4, both arms level, raised and lowered with the aim) the left arm mirrors the
right, and against a frozen NPC (`BF_COMBAT_LOG`, `BF_SHOT_LOG`) 11 pellets from each hand gave 22 hits. `BF_TOOLS_LOG`'s class lines for the other five classes
are the baseline's (main at 968c35c). Not checked: the Berserker's overlay and motions (Kingman's motion
set has none, so none play). The mirrored pose is the demo's own, not a recording of anything (the game has no such clip
for him). Round 3 (`BF_MIRROR_LOG=1`, free cameras from above, in front and at each side, standing and
running, at `BF_CAMERA_PITCH` -0.4, 0 and 0.5, 15 fps frames): both barrels have the same yaw and pitch to a tenth
of a degree (the aim's yaw 0, the barrel's within 2.6 degrees of it, the right gun's own aim from its muzzle), the two
muzzles are level (up 0.00 to 0.01 m), 0.70 - 0.72 m apart across the aim and within 0.02 m of each other along it, and
both guns have their grip down; Tex's berserk frames equal main's (a3e004c) to the pixel.

Checked (#409; `--test` with Kingman alone, `BF_TEST_SQUAD_SLOT="0.3,me,mp_edward_kingman"`, or `BF_CHARACTER=tex
BF_TEST_SQUAD_PRESET="0.3,terrorist_cell"`; `BF_MUTE=1 BF_BACKGROUND=1`, 15 fps `BF_CAPTURE` frames, `BF_TEST_FREECAM` views and the
follow camera turned by `BF_VIEW_YAW`, against main at 3c20f54 where it is "before"; the holster on the thigh and the glides
below are #409's, which #410 replaced: see "Checked (#410)" after):
* **Berserk idle** (`BF_TEST_SKILL=0.3`; front, his side, above): both guns hang at his sides pointing down. `BF_MIRROR_LOG`
  from 1.2 s: the right muzzle (-0.28, -0.60, 8.08), the left (0.29, -0.60, 7.98), level to 0.000 m and 0.008 m from the
  right one's reflection across the torso's plane (the plane follows the shoulders, which the idle clip turns 9 degrees off his
  heading); main's left muzzle was (-0.08, -0.60, 8.23), across his crotch.
* **Walking** (`BF_TEST_WALK=1`), **running** and **the dash** (`BF_TEST_SPRINT`), 80 frames each: the left muzzle is 0.008 m from the
  mirror throughout, the muzzles level within 0.03 m and the barrel pitches within 2.2 degrees. In the frames from the front,
  his side and behind and above the two arms swing as mirrors of each other and both guns are held alike, where main's left
  gun turned about as the heading wandered. A mirror swings both arms forward and back together, in step, as the product
  owner asked ("the right arm's mirror"); a swing in opposite phase would be a different request.
* **Aiming and firing** (`BF_TEST_TRIGGER=0.8-3.0` at `BF_CAMERA_PITCH` 0, 0.5 and -0.4, level, up and down; the free camera drops
  `BF_TEST_FIRE`'s input, `testtools::gate_input`, the trigger hook is applied after it): from the full aim (weight 0.99 on) the two
  barrels' yaws differ by at most 0.3 degrees and their pitches by 0.0, the muzzles are level to 0.01 m, and the lines are main's
  (27, 29 and 30 of the 40 lines from 1.5 s identical to main's; the rest are the aim's weight rising and falling, where the
  plane now eases between the torso's and the aim's). Across the torso's plane the same muzzles are 0.54 m off a mirror: the aim's
  plane is what keeps the barrels parallel.
* **The holster** (idle, not berserk; `BF_WEAPON_LOG=1`): slot 2's gun "stowed at f4ccf78c (bone 1d1b6ac1)", his left
  thigh; seen from his left side and from behind it hangs barrel down on the rear of the outer thigh, its butt and magazine
  turned back (from the front only its barrel's edge shows past the thigh). His right hip stays empty.
* **A swap, the berserk after it, the stow, a second swap** (one run: `BF_TEST_SWITCH=0.4,4.2 BF_TEST_SKILL=1.6,3.0 BF_WEAPON_LOG=1`,
  the log's lines):
  * 0.47 "slot 1 glides from bone 0fb0fa10 (the right hand) to bone 1d1b6ac1 (the left thigh) over 0.25 s, across the front of him"
    and, 0.1 s on, 0.53 "slot 2 glides from bone 1d1b6ac1 to bone 0fb0fa10 over 0.25 s, round his back": the gun in his hand goes
    to the left hip, the one in it comes out into his right hand. Slot 1's place is "stowed at ff4c691b (bone 1d1b6ac1)": place 0,
    the same thigh as place 1's.
  * The berserk at 1.67, its second gun slot 1, the one the swap put away: "slot 1 glides from bone 1d1b6ac1 to bone 057841d8 (the
    left hand)", **drawn from the left hip**, where the first version's gun crossed his body from the right one.
  * The stow at 3.13: "slot 1 glides from bone 057841d8 to bone 1d1b6ac1": back to the left hip.
  * The second swap at 4.33 and 4.40: slot 2, the right hand's, to the left thigh across the front, slot 1 out of it into the right
    hand round his back.

  The frames (close from behind and to his left, from his left, from behind; from above, 15 and 30 fps): the holstered gun stays
  on the rear of the left thigh; at a swap one gun leaves it backwards and round his back while the other comes in from the
  front, so the two are never in one place (at 30 fps from above and at each side: the one going in crosses about 0.25 m in front
  of his coat, the one coming out is 0.3 - 0.45 m behind it, and at the thigh the one leaving is behind it before the one
  arriving is in front); in the berserk the gun rises from the rear of the thigh into the left hand, which closes on it, and the
  stow puts it back, none popping. No gun goes through his coat in the close frames (from his right at 30 fps the one coming
  out of the holster arrives at his right hand from behind, outside the coat all the way).
* **Q pressed in berserk** (`BF_TEST_SKILL=0.3 BF_TEST_SWITCH=1.6`) ends it, the second gun gliding to the left hip, as the
  switch key ends Tex's, and makes no swap.
* **A switch cut short** (the review's point 4: `BF_CHARACTER=flint` alone, `BF_TEST_SWITCH=0.4 BF_TEST_KNOCK=1.0 BF_WEAPON_LOG=1`: the squad's
  own clip, which Kingman's timed switch inherits): main lets the L-Shot-50 go at 0.80 and never takes a gun again, both stowed to the end
  of the run; now at 1.07, the frame after the knock, "slot 1 L-Shot-50: in the hand" (`Player::cancel_switch`).
* **Unchanged**: Tex (`BF_CHARACTER=tex`, berserk on at 0.5 s, off at 3.0 s, a switch at 4.2 s): 80 of 80 frames identical to main's;
  Tex running with the berserk on and a switch: 60 of 60; Brutus, Flint and Hawk (a switch, then a run): 60 of 60 each, and the
  `BF_WEAPON_LOG` placements of all four identical (8, 6, 6 and 6 lines). Tex firing in berserk (1.5 - 2.5 s): 72 of 90 identical,
  the 18 others a few pixels (at most 26 in a channel) round the muzzle's smoke and flash, which main differs from itself in the same
  way (70 of 90 identical between two runs of main: the effects' particles aren't repeatable from run to run).
* `cargo test`: 372 tests pass in the demo and 107 in the library (new: the plane's choice and its blend, the mirrored mounts of the
  hand and the hip, both stow places on the left hip, the glides' start, the frame they start on, arrival, cut-short restart and
  signed bow, the two guns of a swap never within a hand's width of each other, a switch cut short); `cargo build --bins`: no
  warnings. The squad's frames and `BF_WEAPON_LOG` placements against main were run again on the final build (the figures
  above are those runs').

Checked (#410, the product owner's second design; `--test` with Kingman alone, `BF_TEST_SQUAD_SLOT="0.3,me,mp_edward_kingman"`,
`BF_MUTE=1 BF_BACKGROUND=1`; 60 fps `BF_CAPTURE` frames (`BF_CAPTURE_FPS=60`, `BF_NO_HUD=1`) from his left, his right, the front-left,
the front-right, behind and above with `BF_TEST_FREECAM` standing, and the follow camera turned by `BF_VIEW_YAW` (his left, his right,
behind) walking and running; `BF_REACH_LOG=1`, `BF_WEAPON_LOG=1`, `BF_SHOT_LOG=1`, `BF_MIRROR_LOG=1`; against main at c665902. Round 1's
checks, of one holster at the front of his waist, are replaced):
* **A swap standing, right to left and back** (`BF_TEST_SWITCH=2.5,4.0`): "right hand at the hip lets go of slot 1 Kman Auto" at 2.77,
  "left hand at the hip takes slot 2 Kman Auto" at 3.00, the left hand back up with it by 3.25; back, the left lets go at 4.27 and the
  right takes at 4.50. Each hand only goes down its own side: from behind the right elbow bends back to the right hip, then the left
  one to the left hip; from above the elbows go out and back and no forearm crosses his front. On the frame a gun changes hands its
  grip is 0.0000 m and its turn 0.00 degrees from the holster's; no bone of an arm turns more than 5.5 degrees in a frame; the wrist is
  never more than 60 degrees from straight (the idle's own carry; 43 - 45 at the hip); at the hip the arm is 0.79 - 0.82 of its length
  straight, and the clavicle never shrugs more than 0.1 degrees. The holstered grips sit at (0.28, 0.10, 0.14) and (-0.21, 0.07,
  0.13) m in the model's space (his idle leans his hips; the two are mirrors on his pelvis), the gun 0.053 m from the right thigh's
  line at its closest on the way (half way down), outside the thigh in the frames from the front-right and his right.
* **Walking and running** (`BF_TEST_GOTO=0,0,0,-40`, with and without `BF_TEST_WALK`, the swap at 2.0 s and back at 3.5 s): the legs
  keep their walk and run while the hands go to their hips; the hand-over is 0.0000 m and 0.00 degrees; no arm bone turns more than
  6.3 degrees in a frame walking and 16.1 running (a smooth rise and fall over the 15 frames of a leg, as the hand catches up with
  the run's arm swing: 9.9, 12.6, 15.2, 16.1, 15.8, 14.5, 12.7 ...), the wrist at most 43 / 56 degrees from straight, the shrug 0.1.
  Running, the left hand comes up into the run's carry mirrored, the gun held up in front of him as the right one was.
* **Left-handed aim and fire** (`BF_START_WEAPON=1 BF_TEST_TRIGGER=1.0-2.5` at `BF_CAMERA_PITCH` 0, 0.5 and -0.4; from his left and from
  above behind him): his left arm comes up to the aim, the flash and the casings are at the left gun's muzzle, and the shots leave
  from it: `BF_SHOT_LOG` origin (-0.35, 0.93, -1.24) m from him level, (-0.35, 1.48, -0.79) up, (-0.30, 0.49, -1.39) down, against
  (0.34, 0.94, -1.24) with the right gun; level and up the barrel is on the crosshair's ray (0.00, 0.11, -0.99) and (0.00, 0.57,
  -0.82). He stands at a heading of -20 degrees to aim it, against -16 with the right gun (his chest keeps the clips' right-handed
  turn).
* **The berserk draw and stow** (`BF_TEST_SKILL=5.5,7.0`, the right gun in hand): the left hand takes slot 2 at its hip at 5.77 and
  has it up by 6.02; the stow lets go at the hip at 7.27. With the left gun in hand (`BF_START_WEAPON=1 BF_TEST_SKILL=0.5
  BF_TEST_TRIGGER=2.0-3.5`): the right hand takes slot 1 at the right hip at 0.87, then at the full aim both barrels have yaw -0.2 and
  pitch 6.3, the muzzles 0.70 m apart, level and 0.02 m apart along the aim (`BF_MIRROR_LOG`), and each gun's shots leave from its own
  muzzle ("hand: 0" at x -0.35, "hand: 1" at x +0.38). With the right gun in hand the berserk's aim (`BF_TEST_SKILL=0.5
  BF_TEST_TRIGGER=1.5-3.0 BF_CAMERA_PITCH=0 BF_MIRROR_LOG=1`) is main's to the digit from 1.6 s (59 of 59 lines); the log starts at 0.87
  s, when the left hand holds the gun, where main's starts at 0.67.
* **Ground pickups** (`BF_TEST_GOTO=0,0,0,-1.8 BF_TEST_WALK=1 BF_TEST_GUN="Jax-iP,3.0,0.0" BF_TEST_SWAP=4`, the follow camera at the
  front-left): with the right gun in hand "class 0 to slot 1 (slot 1 in hand)", the Kman Auto let go on his right ((0.32, -0.35,
  -1.22) from him) and the Jax-iP "in the right hand" on the grab's frame; with the left (`BF_START_WEAPON=1`) "class 0 to slot 2 (slot 2
  in hand)", the clip's pickup his left hand's, the Kman Auto let go on his left ((-0.39, -0.36, -1.15)) and the Jax-iP "in the left
  hand", slot 1 on the right hip throughout. In berserk with the left gun in hand (`BF_TEST_SKILL=2.0,6.0`, X at 4.0) the use starts at
  4.20 with slot 2, waits through the berserk and the right hand's stow (6.80), plays at 7.20 and replaces slot 2 at 8.00. (The first
  build put the grabbed gun on its holster for the grab's frame, as the hand took it only on the next step: `grab` now gives it to
  the hand at once.)
* **A knock-down in a swap** (`BF_TEST_SWITCH=1.0 BF_TEST_KNOCK=0.95`): the knock cuts the swap after the drop; he gets up with the
  first gun in his right hand and the other on its hip, no hand left at a hip.
* **Unchanged**: Tex, Brutus, Flint and Hawk (`BF_CHARACTER=...`, a switch at 1.5 s and `BF_TEST_GUN="L-Shot-50,3.0,0.0"
  BF_TEST_SWAP=5`, 120 frames at 15 fps each, against main's exe c665902): 120 of 120 frames identical for each, and the
  `BF_WEAPON_LOG` lines identical (20, 16, 20 and 16).
* **After the rebase onto main b33a92c (#211)**: a swap both ways from behind (the same times, 2.77 / 3.00 / 4.27 / 4.50, and the
  same arm turns as above), left-handed fire (the shots from (-0.35, 0.93, -1.24), the flash at the left gun), and Tex, Brutus,
  Flint and Hawk 120 of 120 frames identical to main's exe b33a92c, their `BF_WEAPON_LOG` lines too.
* `cargo test`: 400 tests pass in the demo and 113 in the library on b33a92c (new: a hand's fetch and put-back by the gun the game has it hold,
  a swap holstering on one side and drawing on the other in 0.75 s, a hand at the hip taking back what it let go, the arms' exchange
  and the right arm mirroring the left, a mount mirrored on a bone that is its own mirror, each gun on its own hip; the gun at the
  hip in its mount); `cargo build --bins`: no warnings.

Not checked, or guessed: the reach's times (0.25 s each way, the swap's 0.75 s in all), the bow, the poles and the holsters' lift
(0.12 m up the hip) are the demo's choices and fits to the frames and `BF_REACH_LOG`; no recording of the game shows Kingman
drawing, holstering or shooting left-handed (his motion set has no such clip). Only the arms are exchanged for the left gun: his chest,
head and legs keep the clips' right-handed stance. The fingers keep the clip's own pose (only the arm bones are posed). A reload
with the left gun in hand (his 0.2 s reload clip, mirrored onto the left arm with the rest) wasn't looked at. His model has no
throw clip, so whether a grenade would leave the right hand while the left holds the gun isn't reached. From behind, the game's
camera, the holstered guns show at the sides of his hips.

### Keys and test hooks

* The Confed Marine on the test map: `BF_TEST_SQUAD="3:confed marine@sdm_ferg"` (he takes Tex's
  place and leads), or on any map of a test session `BF_TEST_SQUAD_SLOT="0.3,me,confed
  marine@sdm_ferg"`; the allies the same way (`3:sgt. gonzales@e01`). V or `BF_TEST_SKILL=<s>`
  fires it.
* `BF_TEST_GIVE_SKILL=<skill-set>[,<cost>[,<recovery>]]` gives the character controlled at a map's
  start that skill class (9; the cost and recovery the Marine's 55 and 2.3 unless given, `9,99,0`
  the allies'; 6, the Air Strike, the Militia's 20 and 1), so a squad member fires it on any map.
* `BF_MIRROR_LOG=1` (#407): while a second gun is drawn, both guns' muzzles and barrels in the world (yaw and pitch of
each barrel) and the left muzzle's place from the right's, across the aim, up and along it. #409 adds the aim's weight
(`aim`), the arm's blend (`blend`, 0 - 1 over 0.25 s), the heading of the plane the arm is mirrored across beside the
torso's own and the aim's, and how far the left muzzle is from the right one's reflection across that plane and across
the torso's, each through the point between the shoulders (`MirrorArm::between_shoulders`): 0 for a true mirror.
* `BF_WEAPON_LOG=1` ([weapons](../play/weapons.md)) also prints where each of Kingman's guns hangs (#410: "in the right hand",
  "in the left hand (the second gun)", "stowed at ff4c691b (bone fc373ce8)" for the right hip's and "stowed at f4ccf78c (bone
  fc373ce8)" for the left's, both on his pelvis) and (#410) each time a hand takes a gun at its hip or lets it go there ("right
  hand at the hip lets go of slot 1 Kman Auto", "left hand at the hip takes slot 2 Kman Auto").
  `BF_TEST_SWITCH=<s>[,<s>...]` presses the weapon switch (his swap); `BF_TEST_SKILL=<s>[,<s>...]` the skill (the draw at the first
  time, the stow at the second); `BF_TEST_TRIGGER=<from>-<to>` holds the trigger (it works under `BF_TEST_FREECAM`, where the
  other hooks' input is dropped, `testtools::gate_input`).
* `BF_REACH_LOG=1` (#410, play.rs `reach_log`): each frame one of Kingman's hands is on its way to or from its holster, which
  hand, how far along it is (0 its pose, 1 at the hip), the gun's grip as the hand holds it and at the holster and how far apart they
  are in metres and degrees (0 and 0 on the frame a gun changes hands; "(the hand empty)" after a let-go and on the way to take
  one), the elbow, the wrist's turn from straight, how far the upper arm, the forearm and the hand turned since the last frame (a pop
  is a jump there), the grip's and the gun's least distance from the lines of his thighs and spine, how straight the arm is (1
  straight) and how far the clavicle shrugs.
* `BF_SKILL_LOG=1`: each press ("orbital beam: stamina -55 -> 45.00", "refused: stamina 53.74
  under its cost 55"), the target point and how it was found (`Lock`, `Terrain`, `Face`, `Below`,
  `Nothing`; the camera and the reticle's point); `BF_DAMAGE_LOG=1`: the effect started (its owner
  and rules) and every tick; `BF_RETICLE_LOG=1`: the lock.

### Verified (#200)

`cargo run --bin bf_play -- --test` with `BF_MUTE=1 BF_BACKGROUND=1`, 15 fps captures
(`BF_CAPTURE`; 30 fps with `BF_CAPTURE_FPS=30` where a tick's size matters), `BF_DIFFICULTY=0` (the
deathmatch takes have no difficulty step; the test map's STANDARD gives an enemy x1.4), frames
looked at and laid side by side with the takes:

* **Take01 staged in the Arena** (`BF_MAP=mp1 BF_TEST_SQUAD_SLOT="0.3,me,confed marine@sdm_ferg"
  BF_TEST_GOTO=-15,-24.05,-15,-24.05,20 BF_TEST_SWEEP=180 BF_TEST_NPC="0.5,mp_tex,enemy,frozen,-14.58,-9.01,19.6"
  BF_TEST_NPC_HP=1.0,1,999 BF_TEST_INSTANT_KILL=0 BF_TEST_SKILL=2.5`): the lock, the point
  (-14.57, 20.12, -9.01) (take01: (-14.581, 20.124, -9.027)), 100 -> 45 at the press, the name 0.2 s,
  P2 30.0 at once then 15.0 over 1 s from +2.0 s (take01: 29.61 and 14.72). The picture beside
  take01's, aligned at the press: the flash on P2, the flood of the corridor's end, the orange column
  with its diagonal shafts from +0.7 s, the second flare near +2.5 s, the fade, gone by +3.1 s, as
  the take's; the demo's flood is whiter and its column paler than the game's orange (the ALE
  runtime's colours). The Marine stands 5 cm behind take01's spot: at the spot itself his root is just
  under 15 m from the point (the log's "at 15.00 m") and the 15 m area hurts him (the open edge
  below).
* **The allies** (`BF_TEST_SQUAD="3:sgt. gonzales@e01" BF_TEST_SKILL=1.5,3.5`): 100 -> 1 at the first
  press, no refill (recovery 0), the second press refused at 1.00: one beam a life.
* **Take02, no lock** (the same, no NPC, `BF_TEST_GOTO=-15,-24,-15,-24,20`): the point on the
  corridor's floor (-15.00, 18.93, 0.14), 24.1 m ahead (take02: 23.2 m, the demo's camera pitched a
  little higher), found as an object's face (mp1's floor is a static object: no rise, as take07
  shows); the sound h_f76d4757 the frame after; the strip beside take02's matches it frame for frame
  as take01's does.
* **Take03, the wall** (`BF_TEST_GOTO=-15.4,-24,-15.4,-24,20 BF_TEST_SWEEP=90`): the point on the wall's
  plane x = -20.30 (take03: (-20.30, 20.58, -23.85)), 5.2 m from the feet; the Marine, the owner,
  at 4.95 m (his root) takes 30.0 then 15.0 (take03: 20.82 in two ticks, then 15.04) and stands
  through both, every tick at 4.95 m (take03's P1 stays in motion state 19): a character a player
  controls takes no AI hit reaction (its brain is off, play_enemies.rs `damaged`); with Tex as the
  owner (`BF_TEST_GIVE_SKILL=9`, 30 fps) 30.0 in three ticks of 10, then 15.0.
* **The 15 m radius on the owner** (test map, the Marine, `BF_TEST_NPC="0.5,mp_tex,enemy,frozen,0,-6.5"`,
  locked 14.5 m ahead): he takes 15.0 over the 2 s area's second, every tick at 14.50 m, and nothing
  at the press (past 5 m); standing.
* **Take12, a teammate** (test map, `BF_TEST_GIVE_SKILL=9 BF_TEST_GOTO=0,8,0,8
  BF_TEST_NPC="0.5,mp_tex,friend,frozen,0,-5.8"`): the crosshair green, no lock, the point on his
  body (Face); he takes 30.0 then 15.0 (take12: 28.56 and 15.00) and the owner, 13.5 m off, 15.0 and
  nothing at the press (take12: 15.00 at 13.8 m).
* **Take04, the gate** (test map, the Marine, `BF_TEST_SKILL=2.5,6.3,12.5,17.0,20.5
  BF_TEST_STAMINA=100@19.0`): fired at 100 (-> 45), refused at 53.74 (take04: 53.2-53.6), fired at
  68.0 (-> 13.0; take04 67.03 -> 12.16), refused at 23.27 (take04: 21.6-22.1), fired at 100; the
  refill 2.32 a second (take04: 2.31-2.35); no name and no effect at the refused presses (the name's
  box empty in every frame round them).
* **The name**: in the press's frame, at full for 0.2 s (three 15 fps frames), then fading; the
  takes show it faint at T0 + 0.4 s in both strips, as the 3 a second fade from 0.2 s gives (the
  spec's "gone by 0.30 s" is where the flood swamps it).
* **The Marine's LASER factor**: x1.000 in every owner tick (`BF_DAMAGE_LOG`'s "type 8 x1.000"): his
  `<shield>`'s Type 8 0.6 is the shield's charge rate, and `CharacterType::vulnerability` reads only
  the combat target's own list.
* Unit tests (`cargo test`): the target point's branches (a lock, the ground with and without the
  rise, a wall, a face looking down, the sky), the spawner (one effect at the point, the scatter's
  square, none without an effect), the toggle (refused under the cost, -55, the hold, never on), who
  gets the class, the hook's value, and the `<effect-data>` record read (`bf/character.rs`).
* **Unchanged** (the build before #200 against this one, the logs line for line): sdm_e34's
  radiation barrel (238 lines), e34's rack and its chain (529), e01's explosive barrel (640), a
  thrown Frag's blast, and the four squad skills switched on and off
  (`BF_CHARACTER=<tex|brutus|hawk|flint> BF_TEST_SKILL=1.5,5.0 BF_SKILL_LOG=1`: 26, 33, 27, 25 lines).
  A thrown Molotov's cloud now starts its clock in the explosion's frame and measures to the root
  nodes (the game's rules: [effects.md](../effects.md), "Verified").

### Open and not done

* **The 15.0 m edge**: take01's and take16's owner, 14.98 m (horizontally) from the point, wasn't
  hurt; the code hurts him unless his root stood 2 cm or more back from his position in his idle
  (the analyst can't get the Marine's offset from the data). The demo's root is at the feet's x and
  z, so its Marine at take01's spot is hurt by the 15 m area.
* **That the first ray starts at the camera**: inferred (`[view+0x10]`'s +0x48 object, answers.md Q4).
* **A face looking down**: the demo's bodies are upright cylinders, whose faces never look down, so
  take12's point (the floor 1.2 m before the teammate, off a downward-facing part) is on his body in
  the demo. The rule is in and unit-tested.
* **A squad type's knock-down by a hard hit** (play.rs `KNOCKDOWN_DAMAGE`, the demo's rule, 15 HP a
  hit): a 300 a second area's tick at the 15 fps capture is 20 HP, over it, which at 30 fps (the
  game's) it isn't, so a squad member as the owner can be floored by his own beam in a 15 fps
  capture.
* The allies: the skill is in their types (cost 99, recovery 0); nothing fires it (no personality
  posts goal 127, and the demo's goal refuses an attack skill). O5 (a live ally) wasn't filmed.
* The rumbles (no pad vibration), the effect sound's 3D law (its volume is the effects' blast
  volume), and the beam's height (off-frame in every take; the ALE data's).
