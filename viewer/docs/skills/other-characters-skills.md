*Part of [skills/](README.md), the former page `skills.md`.*

## Other characters' skills: the effect skills and the Orbital Beam (#200)

Beyond the squad's four, a character type's `<combat-skill skill-set>` picks one of 15 more skill
classes (the factory FUN_0012e590; the analyst's map is `todo/npc-skills/answer.md`). The
**effect skills** are a family: the Orbital Beam (CSKILL 9), the Air Strike (6, #203), the Wrath
of Garner (13, #208), the bosses (17, 18, 21) and the Molotov Mutant (20) each start an effect type
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
Edward Kingman class, Tex's Berserker (skill-set 3, #407, below) the only classes a type of its own
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
A Berserker on a type of its own has no berserk motion set or toggle overlay clips (they are
ms_tex's, found by the squad character's name): the press tops the other gun up, draws it and
drains the stamina, the character keeps its own motions. **The second gun and the left arm (demo-own,
the product owner's choices):** only Tex's model has the SECOND_HAND hardpoint h_15ab6d29 and berserk clips.
For a model without one (`play_mirror_arm.rs`, `MirrorArm`): the second gun is held on the **left hand at the
mirror of the trigger hand's mount**, and while it is drawn the character step (play.rs) poses the **left arm as the mirror of
the right** after the aim has turned the right one to the crosshair, so the left gun follows the aim and the
recoil. The bones are the Biped's `Bip01 R|L` `Clavicle`, `UpperArm`, `Forearm` and `Hand` (the names of
play_damage.rs `HIT_BONE_NAMES`; Kingman's skeleton has all eight, both clavicles hanging from one spine bone).
The reflection is made **across the aim's vertical plane**: the plane through the point between the
shoulders that holds the right gun's barrel direction and the up axis (round 3 of #407: the first version
reflected across the torso's own plane, but a standing body stays about 16 degrees off the aim and the mirror
doubled that, so the left barrel splayed 25 - 30 degrees outward and the hand sat farther from the body). A bone's
frame `W` reflected with the matrix `M` is `M W K`, `K` a constant reflection in the bone's own axes that turns the right
bone into the left at rest (`K = Rr0^T M0 Rl0`, `M0` the bind pose's symmetry plane, from the shoulders in the bind
pose), so each left bone is `M Wr K`: with the barrel in the plane its reflection is itself, so both barrels are
parallel to the aim, the hands are level and the same distance from the aim line, and only a shift along the aim of
the shoulders' offset from the plane remains. The left gun is held at `(K^T r S, K^T t)` for the right mount
`(r, t)`, `S` the reflection of the gun's own lateral axis (the one of its two axes off the barrel that the muzzle
has no offset from the grip along), which keeps its barrel and its up: the left gun has the right one's orientation,
grip down. A barrel within 78 degrees of the vertical uses the torso's plane instead. A model without those bones holds the gun at its SUPPORT_HAND
h_f8474385 (Kingman's compound h_e0b89937 has it, objects-m03_c.xml 9868), and one with none of the three
can't draw a second gun: `skills::other_gun` refuses, the fire step's second gun needs a mount, so there is no
phantom fire. Tex's own berserk is untouched (his model has a SECOND_HAND, so he has no `MirrorArm`). The pose
snaps on at the draw and off at the stow (no blend).

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

### Keys and test hooks

* The Confed Marine on the test map: `BF_TEST_SQUAD="3:confed marine@sdm_ferg"` (he takes Tex's
  place and leads), or on any map of a test session `BF_TEST_SQUAD_SLOT="0.3,me,confed
  marine@sdm_ferg"`; the allies the same way (`3:sgt. gonzales@e01`). V or `BF_TEST_SKILL=<s>`
  fires it.
* `BF_TEST_GIVE_SKILL=<skill-set>[,<cost>[,<recovery>]]` gives the character controlled at a map's
  start that skill class (9; the cost and recovery the Marine's 55 and 2.3 unless given, `9,99,0`
  the allies'; 6, the Air Strike, the Militia's 20 and 1), so a squad member fires it on any map.
* `BF_MIRROR_LOG=1` (#407): while a second gun is drawn, both guns' muzzles and barrels in the world (yaw and pitch of
each barrel) and the left muzzle's place from the right's, across the aim, up and along it.
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
