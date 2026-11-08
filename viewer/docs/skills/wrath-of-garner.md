*Part of [skills/](README.md).*

## The Outcast Shaman's Wrath of Garner (#208)

CSKILL_WRATH 13, "Wrath of Garner": the effect skill of the deathmatch's Outcast Shaman (h_e973af9e,
objecttypes-dm) and the Feral Hunting Pack's (h_130090e3, objecttypes-sdm_feral): stamina cost 33,
recovery 2.8, 70 HP. It calls a blast on the crosshair's target and holds the Shaman in a `spirit`
pose, with the skill on for 3 s, the stamina draining all the while and the character not free to
act. The spawner, the target point and the damage areas are the Orbital Beam's ([Other characters'
skills](other-characters-skills.md), #200); this page is the Wrath's own part, `src/bin/play_wrath.rs`
(no systems of its own: a `SkillKind`, as Tex's and Flint's are, dispatched by an arm in each of
play_effect_skills.rs's three family points: `kind`, `own_type_skill`, `give_defaults`). From default.xbe (the rows are in
[code-map/skills-and-stamina.md](../code-map/skills-and-stamina.md): FUN_0013f1c0, FUN_0013f240,
FUN_0013f2f0, 0x13b980, the vtable 0x39c378 and the command handler's return 2), the analyst's
answers (`todo/200-effect-skills/answers.md` §5 and §8) and the takes 08-11 and 18 in
`todo/200-effect-skills/` (the spec is `spec.md` there, §4).

### The class and its toggle

The constructor FUN_0013f1c0 copies game-options' block h_085b2f26 (the hash of `wrath-of-garner`;
`bf/character.rs` `WrathSkill`): its `<effect-data>` (one effect h_1e72e858, no scatter, no pitch,
reach 50, rise 0: FUN_0013de50's copy, the Orbital Beam's record with another effect) and its
`attack-duration`, 3 s (settings +0x4d0, skill+0x40). The drain is the base class's (0x13c690:
`stamina-cost` x dt, 33 a second: `SkillKind::drain`'s default), and the type's start and end sounds
play through the shared on and off.

The toggle FUN_0013f240 (`Wrath::toggle`):

* **On**: it returns 0 at once, so a second press does nothing, nor does the command the stamina
  update sends at 0 (take08's stamina ends at 0.22: the skill ends by its timer alone).
* **Off**, it refuses unless the character has no motion request pending (char+0x748 bit 3), has a
  motion state, has a current MOTION with a motion type 45 transition, has the overlay slots free
  (Tex's toggle's tests) and has a stamina of at least `stamina-cost` x `attack-duration`: 33 x 3 =
  **99**. The compare passes on equal (`fcompp` / `test ah, 0x41` / `jp` at 0x13f2c8-0x13f2cf: the
  code map's earlier "equality refused" read the C's `<` and `==` the wrong way). Take09 (stamina
  written to 92, a tap every 0.3 s): refused up to 98.87, fired at 99.66; takes 08, 10, 11 and 18
  fired at 100. In the feral set (ms_feral_colonist.xml) the idles, the walk, run and dash, the back
  and side steps, the crouch idle and walks and the stand-to-crouch and back have the type 45
  transition; the jumps, dodges, the dive, the slide, the throw, the stun and the knock-down have
  none (`has_transition`: standing, kneeling down or up or kneeling, not sliding or knocked down,
  no grenade in the hand).
* **Accepted**, in this order: the spawner FUN_0013e410(0) (the blast starts in the press's own
  frame: `Skill::fire`), the activate FUN_0010e470 (the 10% check, on, the start sound, the line),
  the timer (skill+0x3c) = the attack-duration, and the return **2**. The command handler turns a 2
  into a request for motion type 45 (0x1224ee: `mov [esp+0xc], 0x2d`), MT_TOGGLE_SPECIAL, which is
  a whole-body motion of the character's motion set, where a 1 queues an overlay (`Toggle::Motion`
  here, [The toggle](the-toggle.md)).
* **The press's frame** (take08): the on bit and the target point in the same sample, the stamina
  still 100.00; the next sample motion state 45, the timer at 2.93 and the stamina 97.78 (two
  frames of 33 a second). In the demo the press's frame takes its first drain and its first time
  off the timer, as FUN_0011a840 orders them after the command.

### The pose

The MOTION h_e327688a (motion type 45; script `spirit`, prefix-type 1, so `Sc_w1_spirit` /
`Sc_w2_spirit` by the held weapon's stance, Brutus's clips, which the ferals' animation set plays:
1.50 s, no events; `aim="false"`, `block-overlay="true"`, h_f6a410e6 `true`, next motion the idle).
Take08's motion state is 45 for 1.45-1.59 s of the log's clock (about 1.5 game seconds). The demo
plays it as `Toggling::pose` (play_skills.rs): the clip laid over the whole body, root included
(`apply_overlay` as the weapon pickups' clips are), from `Loaded::pose_clips` (the type's MT 45
MOTION resolved by stance; a squad member given the skill by the hook falls back to his own
`spirit` overlay, Brutus's, the same clip). While it plays:

* **He stands where he is**: the MOTION's h_f6a410e6 is +0x251 bit 5, which FUN_001195c0 reads to
  skip the stick's move and turn (`skills::holds_still`, `step_player`'s `moving`; a pickup's clip
  holds the same way, #293). A run into the press stops for the pose and goes on after it.
* **No dodge or jump** from it: the MOTION's only transition is to motion type 21, the fall, so
  `step_player` drops them while it plays; a crouch is dropped for the whole 3 s (below).
* The gun isn't aimed at the crosshair (`aim="false"`, the demo's rule for toggle overlays).
* A knock-down or death cuts it (`step`), the skill staying on for its timer through a knock-down.

### Staying on and ending (FUN_0013f2f0)

The skill's update, +0xc0, runs after the stamina update's drain step in the same frame
(FUN_0011a840: the drain, the skill's update, the refill if it's off now): while on, a timer at or
under this frame's time **ends it** (FUN_0010e3e0: the end sound, the line; timer 0; the compare at
0x13f30c passes on equal), else the frame's time comes off it. So it is on for 3 s and one frame at
most: **90 frames at 30 Hz** (take08: the on bit read set from gt 2.413 to 5.347, 2.934 s, each
end +0.05 s, so 2.93-3.03 s of the log's clock, which ran 1.0127 game seconds to the second:
2.97-3.07 game seconds). The drain, taken in each of its frames, leaves 100 - 99 = about 1 (take08
0.22, take10 1.05, take09 1.15; the off frame refills at 2.8 a second as the rule has it: take08
+2.82-2.83). The demo ends it
through the toggle command, the one way its skills switch off: the update asks for it and the
toggle, finding the timer out while on, answers `Toggle::Now`.

### Not free to act (+0x14, 0x13b980)

FUN_00117210, whether a character is free to act, ends by asking the skill's +0x14, which is true
only while the skill is off (the Orbital Beam's and Air Strike's say yes always: 0x213690). So for
the **whole 3 s**, not just the pose's 1.5, `lets_act` is false and each of these asks it:

| Refused while on | Where (the game's call; the demo's) |
|---|---|
| a weapon switch | FUN_0011e130 at 0x11e135; play.rs `start_switch` |
| a reload (a reload asked for starts after the off, as after Flint's) | FUN_00120cd0 / FUN_00120d40 at 0x120e16; play_ammo.rs |
| every item use: the medkit, grenades thrown or set down, the OrgSen | FUN_001210c0 at 0x121139, FUN_00121170; play_pickups.rs, play.rs `can_throw`, play_itembox.rs, play_vision.rs |
| a weapon pickup's use update and the X hold that starts a use (a lying gun, a console or a panel) | FUN_00120290 at 0x1207b9; play_weapon_pickups.rs, play_usables.rs (#208: the hold doesn't arm) |
| the crouch command | the player controller 0x1332a0 at 0x134df6 sends CT_TOGGLE_CROUCH only while free; play.rs `step_player` (#208: a Z pressed while on is dropped, the crouch as it is kept; the AI's own crouches are the brain's, not this command) |
| idle overlays (FUN_00123340's set at char+0x760, read as the idle and taunt overlays) | the demo has none |

Not gated: the stick's move and turn after the pose, firing (the controller's fire section 0x134531
posts CT_FIRE_WEAPON without asking) and the White button, which answers 0 at once while on.
Flint's targeting has the same +0x14 (the same 0x13b980, [Flint's Advanced
Targeting](flints-advanced-targeting.md)), so the crouch command and the X holds (a use of a
console, a panel or a lying gun) are refused for her too while it's on, as the code has them
(#208 added them: before it only the reload, the switch and the item uses asked).

### The effect

h_1e72e858 (common's objecttypes line 592): the ALE effects `exp_hbomb` (an `xp_herm` sprite,
alignment 2), `exp_hbomb_flash`, `_flash2`, `_ring` and `_smoke`, no light-effect; the sound
h_e866e1d9, which plays whole, 4.35 s (the record's `play-length` 1.75 only picks the path and the
window: the Q6 answer); two damage areas **650 EXPLOSION a second for 0.1 s within 5 m at once, then
30 a second for 1 s within 15 m from 2 s**, no falloff, push 0.2. Rates, not lumps (#200's reading
of FUN_0021ade0): two or three frames of a 30 Hz clock in the first area, so 58.7-64.2 HP in the
takes (take08 58.66, take09 59.60, take11 63.76, take18 64.22) and 29.9-30.4 in the second, 88.7-94
in all. The areas reach anyone's root node within range, the owner and his team too (take11: the
teammate 63.76 + 29.86, the owner at 14.8 m 29.86; none at 15.00 m in takes 08, 09 and 18), and the
kill is the owner's.

### The start and end sounds

The type's `start-skill` ee753485 (Brutus's 5.98 s rumble) at the on and `end-skill` 190cd1f6 (0.98 s)
at the off, 2D for the controlled character, as the squad's skills' (`skills::switched`). The takes
(08, 10, 11, 18) hear neither under the effect's own 4.35 s boom, and nothing of the rumble after
it. The analyst (answers.md Q5) reads FUN_0010e750, the first thing FUN_0010e7a0 does (it releases
the skill's current voice with 0 if it still plays), as the end sound cutting the start sound, so
that the rumble ends at the off, inside the boom. #35 take08 measured Brutus's end sound, the same
call, not cutting his 6.1 s start sound (the rest of it still matches after the off,
[Sounds and chatter](sounds-and-chatter.md)), so what the release flag 0 does is open and the demo
keeps the squad's behaviour: **the start sound plays on past the off.** Either way the
controlled character only: whether FUN_00112790 passes P1 of a two-player deathmatch is the
other thing that could keep both sounds out of the takes.

### Keys and test hooks

* V, or `BF_TEST_SKILL=<s>[,<s>...]` (`BF_SKILL_LOG=1`: the toggle, its refusals and why, the pose,
  the target point, the effect started, the off).
* The Shaman himself is any squad slot of a test session that holds his type:
  `BF_MAP=mp1 BF_TEST_SQUAD_SLOT="0.3,me,h_e973af9e"` (the deathmatch type; `h_130090e3` the Feral
  Hunting Pack's), whose `<combat-skill skill-set="13">` gives him the class. His model, clips and
  motion set load as any type's (#394); the Arena (`mp1`) is the takes' map.
* `BF_TEST_GIVE_SKILL=13[,<cost>[,<recovery>]]` gives the class to the character controlled at a
  map's start (cost 33 and recovery 2.8 unless given): Brutus (`BF_CHARACTER=brutus`) stands in with
  his `spirit` overlay, the same clip, for the gating checks on the test floor.
* The takes' staging (take08, W1): `BF_MAP=mp1 BF_TEST_SQUAD_SLOT="0.3,me,h_e973af9e"
  BF_TEST_GOTO=-15.07,-24.29,-15.07,-24.29,20 BF_TEST_SWEEP=180
  BF_TEST_NPC="0.5,mp_tex,enemy,frozen,-14.90,-9.29,19.6" BF_TEST_NPC_HP=1.0,1,999
  BF_TEST_INSTANT_KILL=0 BF_TEST_SKILL=2.5 BF_DIFFICULTY=0` (the deathmatch takes have no difficulty
  step); `BF_TEST_NPC_HP=1.0,1,70` is take18's real 70 HP; `friend` in place of `enemy` is take11's
  teammate; no NPC is take10's floor.

### Verified (#208)

`bf_play --test` with `BF_MUTE=1 BF_BACKGROUND=1`, 30 fps captures (`BF_CAPTURE_FPS=30`), the frames
laid beside the takes' (the takes' P1 band, 960 x 360 of the 60 fps video, the demo's frame
cropped to the same shape), aligned at the press (the take's name first shows at video 2.683 s):

* **W1, locked at 15 m** (take08): the point (-14.89, 20.12, -9.29) = the lock's hips (take08
  (-14.90, 20.12, -9.29)); the name from the press's frame, full for 0.1 s after the off and gone
  0.43 s after it (take08: gone about 0.35-0.45 s after the off in the picture); Tex 999 HP takes
  3 x 21.67 = 65.0 in the press's frame and the two after it (take08 21.7 a frame, 58.66 in 2.7
  ticks) and 30 x 1.0 = 30.0 from +2.0 s (take08 30.05); the Shaman at 15.00 m untouched (take08,
  09, 18); on from 2.57 to 5.57 s, 90 frames; the stamina 100 -> 83.5 at +0.5 s (take08 82.4 on a
  clock 1.3% fast), -33 a second, ~0-1 left at the off, +2.8 a second after. The pose 1.5 s, the
  Shaman held still under a run into it. The glow, then the white-out with its two peaks, the mean
  luma of the whole picture against the pre-press picture's: +0.73 s (3.5) and +1.13 s (3.5) in the
  demo, +0.73 (3.8) and +1.13 (4.0) in take18 and +0.77 (4.1) and +1.20 (4.0) in take08; then at
  +1.4-2.2 s the takes hold about 3.1 (an orange fireball over the doorway and the walls, still
  strong at +2.1) where the demo falls from 2.3 to 1.5 (see below).
* **W1b, the real 70 HP** (take18): 65.0 in the first 0.1 s leaves 5.0, dead 0.2 s into the second
  area (take18: 5.78 left, dead at 0.2 s), the kill line the Shaman's.
* **W2, the gate** (`BF_TEST_STAMINA=92@0.3 BF_TEST_SKILL=2.4,2.7,3.0`, Brutus with the hook):
  refused at 97.79 and 98.72, fired at 99.65 (take09: 98.87 refused, 99.66 fired).
* **W3, no lock** (take10): the floor 24.9 m ahead (-15.07, 18.93, 0.63), found as an object's face
  as the Orbital Beam's take02 is; take10's floor was 28.45 m ahead (its aim a little lower).
* **W4, a teammate** (take11): the crosshair green, no lock, the point on his body (-15.07, 19.92,
  -9.65) (take11 (-14.91, 20.00, -9.51)); he takes 65.0 + 30.0 (take11 63.76 + 29.86) and the
  Shaman, 14.64 m from the point, 30.0 from the second area and nothing from the first (take11
  29.86).
* **Not free to act** (Brutus with the hook on the test floor, Tex for the X hold; the Wrath at 1.0 s,
  on until 4.07):
  the weapon switch at 1.5 s and again at 6.0 s: only the second switched (its sound at 6.13 s);
  the reload at 2.0 s started at 4.13 s, in the off's next frame; Z at 2.5 s ignored, at 6.5 s the
  crouch (the refill x2 from 7.5 s); a medkit at 2.0 s not used, at 6.0 s used; a grenade held from
  2.0 s not thrown, from 6.0 s thrown; X held from 2.0 s on a lying gun nothing, from 6.0 s the
  pickup; V at 2.0 s and 3.0 s: `Refused`; the trigger held through it fired.
* **Unchanged**: #200's take01 staging, the baseline build (968c35c) against this one: 72 log lines
  each, identical but for the order of one background thread's line, and the captures' frames
  within 0.04 of 255 in mean difference. The squad's four skills switched on and off
  (`BF_CHARACTER=<tex|brutus|hawk|flint> BF_TEST_SKILL=1.5,5.0 BF_SKILL_LOG=1`): the logs of the two
  builds are line for line the same.
* **Rebased onto main 094042c** (#203's Air Strike, #202 and #209's ferals' spirits): W1 and W4 give the
  same numbers (65.0 + 30.0, on 3.0 s, the stamina 83.5 -> 1.0, the owner 30.0 at 14.64 m); the
  Orbital Beam, the Air Strike's A1 staging, Brutus's Vengar charge and the four squad skills' logs
  are line for line the baseline's (bf_play_094042c.exe), save nine "foreign part N#id: its own
  material back" lines of the Vengar log, which carry entity ids and differ between two runs of the
  baseline itself (the same nine lines, in another order), so they are run-to-run noise.
* **The owner's red tint in the second area** (W4, take11; `AREA_TINT_RATE`, [effects.md](../effects.md)):
  take11's P1 is the owner, hurt 30 HP a second from +2.0 s (no difficulty step on the Arena: 30.0 at
  the victim). Its frames show **no red wash** at the start of the damage or after it: the picture is
  the orange fireball (the mean of R - (G + B) / 2 over the band, +2 to +5 above take08's, whose owner
  isn't hurt, is the fireball's take-to-take variation: it is there from +1.8 s, before the damage;
  the step at +2.10 s is a dark red quad of the effect coming into view at the top), and what the
  damage does show is the HUD: the health bar's brighten at +2.10 s and the Medkit label going from
  red to white at +2.17 s (health no longer full). The rule as #203 left it (30 a second or more,
  main 094042c) gave the demo a full-screen deep red flash from the area's first tick (frame 137,
  +2.0 s: the whole frame's red excess R - (G + B) / 2 21.3 against W1's 1.7, green over red 0.22
  against 0.96, mean luma 13.3 against 24.7; gone by +2.3 s). Since the test is strict (more than 30,
  take11 being the first take measured at exactly 30.0) the W4 owner has none: at that frame 2.2,
  0.95 and 26.0 against W1's 1.7, 0.96 and 24.7, the frames beside take11's with no red in either, and
  his damage log (1.00 a tick at 30 Hz from t 4.57, the area at 14.64 m) line for line the same. W1
  (the owner not hurt) is unchanged, frame for frame.
* Unit tests (`cargo test`): the gate (98.87 refused, 99 passes), the transitions and busy
  refusals, the timer (90 frames at 30 Hz, 180 at 60), not free to act while on, who gets the class,
  the hook's defaults, the block's shipped values and `holds_still`.

### Open and not done

* **The orange fireball from +1.3 s**: the take has a saturated orange mass over the doorway and its
  walls to +2.2 s and a dim orange wash to +3.4 s; the demo's cloud is a pale band at the top of the
  opening and a grey smoke that lingers to +4.3 s. The first 1.2 s match, so the spawner's point and
  clock are right; the difference is in the ALE runtime's reading of this effect:
  `exp_hbomb.app` (xp_herm, facing 2, angle fade, 9-15 m) and the smoke's emitter (its Frequency
  keys 100 -> 0 at 0.85 s, then 44 a second from 0.9 to 1.23 s, at an emitter that rises 4.9 m).
  Treating a keyed curve's constant as its value at 0 (the first segment's 100 -> 0) adds smoke and
  makes the picture darker, not brighter, so the smoke's count isn't it. Not traced; a follow-up for
  the ALE runtime.
* **The tint's edge** is only known to lie between 30 and 80 HP a second at the victim (take11's none
  at 30.0; the barrels' tint at 80, #85 take29): the Wrath's second area is on the none side, and in
  the campaign its 30 is scaled by the difficulty step (the squad on STANDARD x0.6: 18, none; BRUTAL
  x1.4: 42, which the rule would tint and no take has shown). The pound on STANDARD (exactly 30) goes
  with it, unmeasured.
* **The camera**: the takes' dm camera sits close behind the Shaman (his hood fills the lower
  third); the demo's follow camera for a type of its own stands further back, the whole body in
  view.
* **The start and end sounds**: whether the end sound cuts the start sound (above).
* **A reload asked for during the skill** starts at the off (`reload_wanted`, the machinery Flint's
  targeting has); not filmed for the Wrath.
* The Shaman's AI: the demo's goal refuses an attack skill (`attack_skill`), so no AI Shaman fires
  it; the Feral Hunting Pack's AI doesn't either (no personality posts goal 127).
* The pose from a crouch: the feral set has the transition (`Crouched` passes `has_transition`), and
  the demo stands him up as it starts (`start_pose` drops the crouch), where the game's next motion
  is the idle; not filmed.
