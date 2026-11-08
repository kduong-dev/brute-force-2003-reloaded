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
demo-own choice: [his Cloak](ghents-cloak.md). No other class has an override.
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

**Kingman's left hip and the swap (demo-own, #409, `play_holster.rs`; his hands draw and holster since #410,
`play_reach.rs`).** His compound has no stow hardpoint
(its 17 hardpoints are the two hands', the head's and the bones' effect points: objects-m03_c.xml ~9868), so a gun that
wasn't in his hand was hidden. The type that carries the Berserker, on a model with none of the three stow
hardpoints (`weapon::STOW`) or the SECOND_HAND, gets a holster on his **left hip** (`Holsters::new`; every other
model, Tex's ramps and the squad's hips included, is as it was), and **whichever pistol isn't in his right hand
hangs from it**: the two Kman Autos are the same gun, so the one he isn't holding is always on the left hip, and the
second gun always comes from the nearer hip, the one his left hand is at in berserk, the product owner's rule. (The first
commit of #409 gave each gun a hip of its own, the first the right and the second the left, and after a swap the berserk draw
took the gun across his body from the right hip.) Both of the game's stow places the guns' slots start on
(`StowPlaces`: place 0 the first gun's, place 1 the second's) are this one holster; his right hip is never used.
* **The holster** (#409): a stow hardpoint on `Bip01 L Thigh`, the mirror of Flint's, the squad member who wears a
  pistol on his hip (his place 1, h_f4ccf78c on his right thigh, objects-common.xml h_160a1622: point -0.143255
  0.12138 0.087056, ORIENTATION 0.491457 0.464461 0.457451 0.577481; the same mirror as the left hand's mount, with the
  thighs' own `K`, `MirrorArm::left_hip`). Its point is scaled by the thighs' lengths (Kingman's 0.511 m to Flint's
  0.457) and moved 0.10 m back along the thigh frame's z (`HIP_SHIFT`, a fit: Flint's holster sits where a hand
  hangs). The pistols' HOLSTER hardpoints h_10ab40ac (the four in objects-dm.xml and objects-m03_c.xml, the Kman Auto's
  among them) are all turned alike, ORIENTATION 0 0 1 0 (the rifles' another), so the Kman Auto hangs barrel down as
  Flint's LZR-10 does.
* **At his waist (#410).** Once his hands draw and holster (below), the thigh was out of his right hand's reach: its
  grip there is 0.84 m from his right shoulder and his arm is 0.69 m long (upper arm 0.343, forearm 0.346), and
  `BF_REACH_LOG` had the wrist 5 cm short of it even with the shrug and the turn into the draw, the path through the front
  of his left thigh; at the side of his waist the forearm went through his coat. So the holster is moved to **the front of
  his waist on the left**, where a cross-draw holster sits, onto the bone the thighs hang from (`Bip01 Pelvis`, where a
  belt rides, so it no longer swings with the thigh): #409's place as his idle clip's first frame has it, moved 0.05 m to
  his right, 0.22 m up and 0.38 m forward (`WAIST_LIFT`) and turned 20 degrees about the up axis about the gun
  (`WAIST_TURN`), so the gun hangs barrel down just in front of his coat (its grip 0.18 m above his hip joints, 0.15 m left of
  his middle and 0.29 m in front of them; the coat's front is 0.27 - 0.29 m out there). The turn is a fit: the right hand
  comes to the holster across his front and the left from his side, so no one turn of one gun at one place suits both wrists
  as his idle holds them (their natural grips are 133 degrees apart); each hand's wrist turns half the way to the gun's grip
  and the gun turns in the hand for the rest (below), and 20 degrees asks the least of both, about 67 degrees each in all.
  (Held exactly in its mount with no turn in the hand, every turn bent one wrist or the other 108 - 165 degrees from
  straight, `BF_REACH_LOG`; a turn that tilted the barrel too stood the gun out from his belly in frames from above and the
  side.) **From behind (the game's camera) the holstered gun is now hidden by his body**, where #409's showed on the outside
  of his thigh.
* **The swap.** Kingman's motion set has no switch clip, so the switch had been instant. The demo gives him a
  switch of its own with the squad's machinery (`start_switch`, `Switching`; `switch_clips_of`: a clip that animates
  no bone, `SwitchClip::reach`), and since #410 **his right hand does it**: it carries the gun in it across his front to the
  holster, lets it go there at 0.25 s (`SWAP_DROP`, the switch's drop), stays at the hip 0.05 s while it closes on the
  other (`DWELL`), takes it there at 0.30 s (`SWAP_GRAB`) and brings it up to the carry by 0.55 s (`SWAP_TIME`), nothing
  aiming or firing until then (the aim comes after, as after any switch). One hand does both legs, the holster first and the
  draw after it, so the two guns never travel at once. The two Kman Autos are one model, so at the hip, where the hand
  lets one go and takes the other, they lie in one place. (#409 moved the guns on their own, each in a 0.25 s glide, the
  one going in across his front and the one coming out round his back, 0.35 s in all; its tester found the first passing
  through his left thigh for about 30 ms and the second hanging 0.4 m behind his back for about 0.1 s. A gun now moves
  only with the hand that holds it.)
* **The berserk draw and stow (#410).** The Berserker on a type of its own has no toggle clip, so the skill draws the
  second gun the moment it comes on (`skills::other_gun`); the gun stays on the hip while **his left hand reaches forward to
  it** (0.25 s), closes on it, and brings it up into the mirrored pose (0.25 s), and only then does it fire
  (`Reaches::left_gun`; the fire's second gun waits for it). The stow at the skill's end takes it back down the same way,
  lets go at the hip and the arm returns to its clip. The mirrored arm (`Player::mirror`, play_mirror_arm.rs) is wanted
  while the left hand holds the gun, so it eases in as the hand comes up with it and out as the hand comes back empty.
  A switch can't start while the left hand is on its way (both hands go to the one holster), and a pickup's clip waits for
  both hands.
* **The arm** (`reach::pose_arm`): a two-bone IK on `Bip01 R|L UpperArm`, `Forearm` and `Hand` over whatever the clips,
  the aim and the mirror made of the pose, so the legs keep walking or running under it. The gun's grip point follows an
  eased path (smoothstep, `Reach::at` moving 4 a second, `LEG_TIME` 0.25 s) from where the hand holds it to where it is at
  the holster (`holstered`, `reach::Holstered`), bowed out in front of the body by 0.45 of the width the path crosses (`BOW`:
  about 0.25 m at the middle of the reach across his front); the elbow turns from the pose's own bend toward its pole
  (`POLE_RIGHT` out and forward for the reach across the body, `POLE_LEFT` out and back); the clavicle shrugs up to 25
  degrees where the arm alone falls short (`CLAVICLE_REACH`) and for the reach across, the spine above the hips turns up to 20
  degrees toward the far hip (`TWIST_ACROSS`). **The wrist and the gun share the turn**: the hand turns from its own turn under
  the forearm (the clip's wrist) half the way to the turn that holds the holstered gun in its mount (`WRIST_SHARE`), and the gun
  turns in the hand about its grip for the rest, as far along as the leg is (`Reach::hold`, which `weapon_fx` puts it at each
  frame), so at the hip the gun in the hand is exactly where the holster has it and nothing pops when it changes hands. (The
  first version turned the hand all the way to the grip: the wrists bent up to 165 degrees from straight, and where the turns
  passed half a turn the forearm's roll and the hand's turn flipped 96 - 154 degrees in a frame, `BF_REACH_LOG`.) At the start
  of a leg the IK gives the pose back exactly (the target is where the hand holds the grip, the pole the elbow's own bend, the
  hand's turn its own), so the arm doesn't pop either. A switch cut short (a knock-down) between the drop and the grab gives the
  gun back to the hand at the hip (`Player::cancel_switch`), which then eases back.

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

Checked (#410; `--test` with Kingman alone, `BF_TEST_SQUAD_SLOT="0.3,me,mp_edward_kingman"`, `BF_MUTE=1 BF_BACKGROUND=1`; 60 fps
`BF_CAPTURE` frames (`BF_CAPTURE_FPS=60`) from his left, behind, the front-left and above with `BF_TEST_FREECAM` standing, and the
follow camera turned by `BF_VIEW_YAW` (his left, the front-left and the front-right) walking and running; `BF_REACH_LOG=1` and
`BF_WEAPON_LOG=1`; against main at a2ea393 where it is "before"):
* **A swap standing** (`BF_TEST_SWITCH=1.0`): the right hand lifts the gun at his side and carries it across the front of his
  coat (the grip's path 0.3 - 0.47 m from the lines of his spine and thighs, the gun's 0.30 m and more except at the start, 0.107 m
  from the right thigh's line where the idle's carry already hangs it at 0.127), sets it in the holster at 0.25 s ("slot 1 ...
  stowed at ff4c691b (bone fc373ce8)", the pelvis), takes the other there at 0.30 s and brings it back to his side by 0.55 s. On
  the frame the gun changes hands its grip is 0.0000 m and its turn 0.00 degrees from the holster's, both ways; no bone of the arm
  turns more than 10.4 degrees in a frame at 60 fps (the upper arm 10.4, the forearm 10.2, the hand 8.5); the wrist is never more
  than 70 degrees from straight (the idle's 45 - 60) and the gun turns at most 34 degrees in the hand. From behind the hand goes
  forward round his front and back and no gun shows through him; from above the forearm sweeps across the front of his coat.
* **Walking and running** (`BF_TEST_GOTO=0,0,0,-40`, with and without `BF_TEST_WALK`, the swap at 2.0 s and the berserk at 3.0 -
  4.5 s): the legs keep their walk and run while the hand crosses to the holster and back; the hand-over is again 0.0000 m and 0.00
  degrees; no arm bone turns more than 12.4 (walking) and 13.4 (running) degrees in a frame; the gun turns up to 46 - 57 degrees in
  the hand there, as the walk and the run hold the wrist other than the idle does. Walking, on the way back the gun passes 0.060 m
  from the right thigh's line as that thigh swings forward; from the front-right its barrel stays in front of the thigh.
* **The berserk draw and stow** (`BF_TEST_SKILL=2.5,4.0`): the second gun drawn at 2.53 stays on the holster until the left hand
  gets there, "left hand at the hip takes slot 1 Kman Auto" at 2.78, and is up in the mirrored pose by about 3.03; the stow lets go
  at the hip at 4.28 and the hand is back at its side by 4.53. No bone of the left arm turns more than 6.8 degrees in a frame, the
  wrist at most 67 degrees from straight, the gun at most 36 degrees in the hand; the grip keeps 0.22 m from the body's lines.
* **The berserk's aim and fire** (`BF_TEST_SKILL=0.5 BF_TEST_TRIGGER=1.5-3.0 BF_CAMERA_PITCH=0 BF_MIRROR_LOG=1`): from 1.6 s all 59
  lines are main's to the digit (the barrels' yaws within 0.2 degrees and pitches 0.0 apart at the full aim); the log now starts at
  0.87 s, when the left hand holds the gun, where main's started at 0.67.
* **The swap and the berserk under a knock-down or each other** (`BF_TEST_SWITCH=1.0 BF_TEST_KNOCK=1.28`): the knock in the hold at the
  hip gives the gun let go back to the hand there (1.30 "slot 1 Kman Auto: in the hand"), at the turn the hand holds a gun at the
  hip (`Reach::hold` is kept whether the hand holds one or not, as the knock comes after the pose: the first build put it back in
  its plain mount for that frame, 34 degrees off), and the hand eases back over 0.25 s, no bone turning more than 10.4 degrees in
  a frame, as in any leg. Q in berserk (`BF_TEST_SKILL=1.0 BF_TEST_SWITCH=2.0,2.2,3.0`) ends it as before (the
  first press inside the skill's timer is refused, "its timer runs", the game's), the left hand puts the gun back at 2.48, and the
  next Q swaps (3.27 / 3.32).
* **Ground pickups** (`BF_TEST_GOTO=0,0,0,-1.8 BF_TEST_WALK=1 BF_TEST_GUN="Jax-iP,3.0,0.0" BF_TEST_SWAP=4`, the follow camera at the
  front-left): with slot 1 in hand "class 0 to slot 1 (slot 1 in hand, mode 0)", `Sc_w1_drop_pickup_w1` (1.00 s, release 0.10, grab
  0.63), the Kman Auto dropped from his hand and the Jax-iP into slot 1, as main; with slot 2 in hand (`BF_START_WEAPON=1`) **"class 0
  to slot 2 (slot 2 in hand, mode 0)" with no switch**, the same clip, slot 2's Kman Auto dropped and the Jax-iP into slot 2, slot 1's
  Kman Auto on the holster throughout (main switched to slot 1 first and replaced it). In berserk (`BF_TEST_SKILL=2.0,6.0`, X held at
  4.0) the use starts at 4.30 with slot 1, the right hand's, waits through the berserk, and plays at 6.60, after the stow (6.33),
  replacing slot 1 at 7.23.
* **Unchanged**: Tex, Brutus, Flint and Hawk (`BF_CHARACTER=...`, a switch at 1.5 s and `BF_TEST_GUN="L-Shot-50,3.0,0.0"
  BF_TEST_SWAP=5`, 120 frames at 15 fps each, against main's exe): 120 of 120 frames identical for Tex, Brutus and Flint, and for
  Hawk on a second run (the first had one pixel 9 off in one frame, as main's own runs can differ), and the `BF_WEAPON_LOG` lines
  identical (20, 16, 20 and 16, the switch and the pickup each).
* `cargo test`: 387 tests pass in the demo and 108 in the library (new: the reach's legs and the left hand's fetch and put-back,
  a draw called off, the eased and bowed path, a reach that starts on the pose, the gun in the hand exactly at the holster at the
  hip, a reach that turns smoothly; both guns on one place under the waist's bone; Kingman's pickup into the slot in his hand);
  `cargo build --bins`: no warnings.

Not checked, or guessed: the reach's times (0.25 s each way, the 0.05 s hold, the swap's 0.55 s in all), the bow, the poles, the
shrug, the turn into the draw and the wrist's half share are the demo's choices, and the holster's place at the front of his waist
and its turn are fits to the frames and `BF_REACH_LOG`; no recording of the game shows Kingman drawing or holstering (his motion set
has no such clip). The fingers keep the clip's own pose (only the arm bones are posed), and the gun turns up to 57 degrees in the
hand as it nears the holster (the wrist takes the other half of the turn), where a real hand would turn the wrist further. The two
Kman Autos are one model: at the hip, in the swap's 0.05 s hold, they lie in one place; two different pistols (after a pickup)
would overlap there for those frames. A switch pressed while the left hand is still on its way to or from the holster is refused
(both hands go to the one holster); no run pressed it in that half second. From behind, the game's camera, the holstered gun is
hidden by his body.

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
* `BF_WEAPON_LOG=1` ([weapons](../play/weapons.md)) also prints where each of Kingman's guns hangs (the hand, the second
  hand, "stowed at f4ccf78c (bone fc373ce8)": the gun at the front of his waist, on his pelvis, since #410) and (#410) each time
  his left hand takes the berserk's second gun at the holster or lets it go there ("left hand at the hip takes slot 1 Kman Auto").
  `BF_TEST_SWITCH=<s>[,<s>...]` presses the weapon switch (his swap); `BF_TEST_SKILL=<s>[,<s>...]` the skill (the draw at the first
  time, the stow at the second); `BF_TEST_TRIGGER=<from>-<to>` holds the trigger (it works under `BF_TEST_FREECAM`, where the
  other hooks' input is dropped, `testtools::gate_input`).
* `BF_REACH_LOG=1` (#410, play.rs `reach_log`): each frame Kingman's hand is on its way to or from the holster, how far along it
  is (0 its pose, 1 at the hip), the gun's grip as the hand holds it and at the holster and how far apart they are in metres and
  degrees (0 and 0 on the frame a gun changes hands; "(the hand empty)" in the swap's hold and after a let-go), how far the gun is
  turned in the hand, the elbow, the wrist's turn from straight, how far the upper arm, the forearm and the hand turned since the
  last frame (a pop is a jump there), and the grip's and the gun's least distance from the lines of his thighs and spine.
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
