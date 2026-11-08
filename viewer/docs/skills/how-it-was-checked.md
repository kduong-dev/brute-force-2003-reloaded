*Part of [skills/](README.md), the former page `skills.md`.*

## How it was checked

Captures on the flat floor (`BF_MAP=flat BF_TEST_GOTO=0,4,0,4`, campaign rates) and sdm_e34,
`BF_CAPTURE` at 15 fps (60 for the fades), with `BF_SKILL_LOG`, against the takes' watch logs
and frames (scripts in the developer's scratchpad: rates fitted per state, the frames cropped to
the HUD at the same scale).

**Rates** (per s; the takes' logs per second of game time, all 0.8-1.4% above the data, the
rig's time base):

| | data | takes | demo |
|---|---|---|---|
| Tex on / off | -2.2 / +1.8 | -2.231 / +1.822 (take02) | -2.200 / +1.800 |
| Tex crouched, sprinting | +3.6, +0.45 | +3.655, +0.456 (take02) | +3.600, +0.450 |
| Brutus | -2.0 / +1.2 | -2.020 / +1.211 (take07) | -2.000 / +1.200 |
| Hawk | -2.5 / +0.6 | -2.527 / +0.605 (take11) | -2.500 / +0.600 |
| Flint | -3.2 / +0.8 | -3.230 / +0.806 (take17) | -3.200 / +0.800 |
| Tex on sdm_e34 | -6.5 / +1.2 | -6.555 / +1.215 (take21) | -6.50 / +1.20 |

**Timings**: Tex on 0.53 s after the press frame at 15 fps (0.50 at 60), off at 0 0.53 s after
the frame it reached 0 (take02: 0.51-0.56); Hawk on 0.87 s after the press frame at 15 fps;
Brutus and Flint in the press frame, off in the frame of 0, Brutus with no overlay; 9.9 refused
and 10.0 on (the poke and the press in one frame; take03's 9.2 / 11.2). The AI's Tex at 0 after a
hand-over: his stop clip, off 0.53 s later, no sound, its line heard (`BF_SKILL_CHATTER=100`).
Death with Vengar on: off in the frame of death, the end sound, the stamina frozen. V while a
grenade charges (`BF_TEST_THROW=1.0,1.0 BF_TEST_SKILL=1.5`): refused, the grenade thrown. A
knock-down mid-sprint (`BF_TEST_SPRINT`, `BF_TEST_KNOCK`): x1 while down.

**In a cutscene** (#256): the tutorial's opening (`BF_MAP=tutorial BF_SKILL_LOG=1
BF_SCRIPT_LOG=1`) with V tapped once a second from its start, by WM_KEYDOWN / WM_KEYUP posted to
the demo's window (it works under `BF_BACKGROUND`): no toggle from the 26 taps through 25.1 s,
the controls back at 26.26 s, then a toggle on each tap from 26.82 s. The same with
`BF_TEST_SKILL=2,10,20,25.9,27,30` and `BF_CAPTURE`: the first toggle at 27.07, Berserker on
screen from then; e40's opening with `BF_TEST_SKILL=2,5,12,20,23.5,26,28`: the controls back at
24.20 s, toggles at 26.07 and 28.07 only. Regressions: each character's skill (`BF_CHARACTER=0..3
BF_TEST_SKILL=1.0,3.0`) on the test map (`-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=0,4,0,4`) and on
e40 with the squad (`BF_TEST_GOTO=-16,152.32,-16,152.32`): on at 1.07, off at 3.13, as before.

**Chatter** (`BF_SKILL_CHATTER=100`, presses at 1, 6 and 9 s, `BF_TEST_STAMINA=2@13`): each
character said the on line at 1 s and was refused it at 9 s (its replay delay, 1.9 s left); the off
line with stamina left at 6 s was said by Flint only (Tex, Brutus: no variant for 0 in 2..100;
Hawk: 1..100); the line at 0 by Tex, Brutus and Flint, not Hawk (no fight).

**HUD**, side by side with the takes at the same scale:
* Bars at full: x 73.4-263.9 for Tex, 73.4-247.3 for Brutus, as take01 / take06.
* take03 (pokes to 9 and 11, presses at 2, 7, 11 s): the trail's end within 1.4 units of the
  take's at 3, 4.1, 5 and 6 s (245.3 / 221.3 / 205.3 / 185.3 against 243.9 / 221.9 / 204.6 /
  184.6: 20.0 units a second, the take's 19.8), the fill's end within 1.3, white then pale, the
  name, the see-through channel; the blink's red phases from 2.000 / 4.067 / 7.000 / 9.067 s,
  1.00-1.07 s each (the take's 2.050 / 4.117 / 7.017 / 9.050, 1.00-1.05 s; the take's picture
  lags its log by 0.13 s, and at 15 fps each half of the wave is whole frames, 0.53 s).
* take02 red phase: the frame's lower half (41-58, 13-21, 14-36) in the take, (36-63, 4-7, 0-13)
  in the demo over a black sky (the take's green and blue are the wall behind).
* take01 off: pale in one frame; the name full 0.15-0.17 s, then gone 0.33 s later (the take: 0.13
  s, then 0.30 s).
* take25: pale to teal linearly in 1.0 s once full (the take 1.0 s).
* take19: at a hand-over Hawk's white bar and "Stealth" become Tex's full teal bar in one frame.
* The name's letter cores span x 56.7-130.0, y 89.6-103.6 ("Berserker"; take01 56.7-130.0, 90.0-103.3);
  "Vengar" 56.7-108.0 against take06's 55.3-110.0 (the demo's font isn't the game's).

**The Squad Command menu's Toggle Ability** (#333; e34 staged as the #33 takes, Tex controlled:
`BF_MAP=e34 BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999 BF_TEST_NO_HOSTILES=1
BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_SKILL_LOG=1 BF_ORDER_LOG=1
"BF_TEST_COMMAND=2:brutus:0.4;3:toggle;5:hawk:0.4;6:toggle;9:flint:0.4;10:toggle;14:brutus:0.4;15:toggle;17:hawk:0.4;18:toggle;22:flint:0.4;23:toggle"`):
each goal's first command comes in the first frame after the menu's fade-out. Brutus on and off
in that frame, silent and nothing to see, 2.0 a second off his stamina while on, 1.2 back after.
Hawk's clip, on 0.87 s later, faded out over the next second seen from Tex (frames 105-130), and
back at the second order. Flint, kneeling at the order (frames 135-159), stood up (159-163) and
took her sniper motions (from 165); off at the second. With Brutus controlled
(`BF_CHARACTER=0 "BF_TEST_COMMAND=2:tex:0.4;3:toggle;13:tex:0.4;14:toggle" BF_TEST_SKILL=6,9`):
Tex reached for and drew his second gun (frames 56-70), on 0.53 s after the command, both guns
in hand till the second order stowed it; Brutus's own V on and off at 6 and 9 s with its sound,
the Vengar view and the red fade, as before (frames 125, 160). On sdm_e34 (Tex 6.5 a second,
`BF_CHARACTER=0 BF_TEST_SQUAD_SKILL=tex,1,18`): his berserk ran to 0 at 16.93 s and his stop clip
ended it 0.53 s later with the line at 0, then it refilled and stayed off; ordered again at 18.0 s
with 1.4 stamina he stood (x2 to x1) and was refused till 10.08 at 25.33 s, then on. Flint the
same (`flint,1,17`: on at 10.03). The goal's refusals went unlogged; since #333's review its
first one is told, once a goal (`Skill::told`), and a member a test hook holds runs the goal too
(the crosshair recipe below). A refusal that lasts (`--test BF_CHARACTER=3
"BF_TEST_WEAPON=Confed LZR-50;Jax-iC" BF_TEST_CLIP=99,0 BF_TEST_RESERVE=0 BF_TEST_SELECT=0,1.0
BF_TEST_SQUAD_SKILL=tex,3.0 BF_SKILL_LOG=1 BF_ORDER_LOG=2`: Tex handed over to Brutus with his
Jax-iC empty and no reserve, then ordered at 3 s): "berserk top-up of weapon 1: 0 rounds",
"berserk refused: the second gun has no rounds" and the Refused line once at 3.13 s, nothing
after; Tex stood where the goal caught him, 1.1 m from Brutus and short of his place, for the
run's last 7 s.

**Guessed or open**: the fight count for the line at 0 (a guess, [chatter.md](../chatter.md));
Brutus's unheard on line; what holds the name 0.15 s; which message widgets push the name
down; the blink's colour operation (the red copy matches the takes' pixels); the ×4 refill;
Brutus and Flint at the 10% threshold (code only); the demo's choices under "The toggle"; the
demo's frame is ~3 units longer on the right than the game's (the tutorial picture's proportions,
play.md). Toggle Ability (#333): an AI Brutus in Vengar showing nothing is the code's reading
(0x13c038), not filmed; the member standing while its goal waits (a stand-up, its stamina) and
the goal having no time limit are inferred; no take shows Tex's or Flint's toggle under the AI.

### Tex's Berserker (#121)

On campaign e34 at the takes' range spot (`BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-53.69,31.29
BF_TEST_WEAPON="Confed LZR-50;Jax-iC"`, `BF_SKILL_LOG`, `BF_AMMO_LOG`, 15 fps captures), against the
takes' watch logs and frames:

* **take01** (`BF_TEST_SKILL=2,18`): on 0.53 s after the press frame and off 0.53 s after the
  second (the takes' mean 0.537 / 0.527); 100 → 64.80 over the 16 s on (2.2/s; take01 65.52 at
  18.64). The weapon box: one row before, both guns named for 2.5 s from the switch, then two
  rows; the list after the off with the Jax-iC blue, as the take's.
* **take04** (`-- --test`, an e34 Shrike Heavy at the take's place `BF_TEST_NPC="0.5,shrike
  heavy,enemy,dummy,-55.39,37.57" BF_TEST_NPC_HP=1.0,1,999 BF_TEST_INSTANT_KILL=0
  BF_TEST_TRIGGER=2-5,9.5-12.5 BF_TEST_SKILL=7,15 BF_COMBAT_LOG=1 BF_SHOT_LOG=1`): both guns' first
  shots in one game frame; the LZR-50 every 8-9 frames (0.27-0.30 s, 10 shots in the 3 s: take04
  0.292 s, 10 shots), the Jax-iC every 16 (0.533 s, 6 shots: take04 0.516 s, 6). Each gun hits
  with its own damage, as in its normal fire: the Jax-iC 32.9-33.4 on a calf (the take's 30.1-35.2,
  its calf hits) and 63-69 on a thigh, the LZR-50 3.5-4.9 on a calf (the take's 2.8-4.9) and
  6.6-10.3 on a thigh; the burst's damage per second is the two guns' sum (take04's 87.6/s is ten
  LZR hits and six Jax-iC hits at the calf values). In the frames the LZR's orange bolt leaves his
  right hand and the Jax-iC's purple ball his left, both to the crosshair (take04 9.67-9.90 s);
  from behind, the berserk idle's pose as take04 8.5 s's (both guns held out in front, their
  muzzles showing either side of his shoulders from the raised camera; from the front, which no
  take shows, the guns are at his hips).
* **take05** (`BF_START_WEAPON=1 BF_TEST_CLIP=99,4 BF_TEST_SKILL=2,14 BF_TEST_TRIGGER=4-10`): the
  LZR-50 drawn; nothing topped up (it recharges); the drain 2.21/s, then 1.10/s once the Jax-iC is
  empty (~5.8 s; take05 5.84 s, -2.22 / -1.12); the Jax-iC clicks dry (16bbf95c), no reload while
  on; the LZR-50 fires on alone; off at 14.67, the reload at 15.20 with the reserve 200 → 164
  (take05: 15.19 s, 164). The weapon box frame by frame with the take's: "4 / 200", the two rows
  named, "0 / 200" red over "92 Regen" at 6.2 s (the take the same), 78 / 80 Regen at 10.2 /
  13.0 s (the take 79 / 81), "0 / 164" then "36 / 164" with the list after the off.
* **take25** (`BF_TEST_CLIP=2 BF_TEST_SKILL=2,6`): the Jax-iC 2 → 36 (34 from the reserve) at the
  press, before the switch; the LZR-50 not topped up. With every clip and reserve at 0
  (`BF_TEST_CLIP=0 BF_TEST_RESERVE=0`): refused, "the second gun has no rounds".
* **take26** (`BF_START_WEAPON=1 BF_TEST_CLIP=99,20 BF_TEST_SKILL=2,10,18,24 BF_TEST_SWITCH=4.5
  BF_TEST_RELOAD=12.5 BF_TEST_THROW=20.5,0.15`): Q at 4.5 ends it (off 5.01, no switch: take26
  5.08-5.13 after Y at 4.5); R at 12.5 ends it (off 13.01) and the reload follows as the stop clip
  ends (13.52: 16 rounds, 200 → 184, the take's 184); G at 20.5 throws nothing, the count stays,
  berserk stays on to the press at 24. The other gun dry (`BF_TEST_CLIP=4,36`, trigger 4.5-8.5):
  the LZR-50 empty at 6.80, the toggle in that frame, off at 7.33 (take26: 30.23 → 30.73-30.78);
  the box "0 Regen" red, then the list with "Confed LZR-50 0 Regen" in blue, the Jax-iC firing on
  (as the take's).
* **Refusals**: crouched (`BF_TEST_CROUCH=1 BF_TEST_SKILL=3`: "no berserk transition"), scoped
  (`BF_TEST_WEAPON="L-Shot-75;Jax-iC" BF_TEST_SCOPE=1`, without `BF_TEST_GOTO`, whose autopilot
  doesn't read it), no rounds for the second gun (above).
* **Motions** (`BF_MAP=flat BF_AUTOPILOT=1 BF_TEST_SKILL=1.0 BF_MOVE_LOG=1`): from the press on,
  bz_idle, bz_run (the autopilot's sprint stays bz_run), the aimed side step bz_f_side_walk_r, the
  backpedal bz_back_walk, the dodge at 13.0 s a plain run, both jumps bz_jump_launch; the
  autopilot's Q at 21.8 s ends it and the normal set is back at the press. Aimed left
  (`BF_TEST_AIM=1 BF_TEST_STRAFE=-1,0`): bz_f_side_walk_l moved by bz_walk's root (1.3 m/s). No
  take shows these (spec shot T1).
* **Death** (`BF_TEST_DIE=3`): off in the frame of death with the end sound; both guns fall.
  **Hand-over** (`-- --test BF_TEST_SELECT=1,3.0` and a fighting enemy Brutus NPC): the AI's Tex
  stays in berserk, drains 2.2/s and fires both guns (the LZR-23's and the Minigun's hits, ammo
  types 11 and 2).
* **The lock's reach** (`-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.535,19.5
  BF_CAMERA_PITCH=-0.07 BF_TEST_NPC=0.5,tex,enemy,dummy,8,-30 BF_TEST_SKILL=2 BF_RETICLE_LOG=1`,
  the takes' loadout): the enemy 50 m out red with the LZR-50 alone (0.60 s: "locks", 53.6 m
  deep), blue from the Jax-iC's draw (2.60 s: "out of reach: 50.9 m from the muzzle, reach
  42.0"); the frames red at 1.3-2.0 s, blue at 3.0-4.0.
* **Taps** (the same spot, no NPC, `BF_TEST_SKILL=2 BF_TEST_TRIGGER=3.5-3.75,3.98-4.06,6.0-6.08
  BF_SHOT_LOG=1`): the hold's first shots both at 3.80; the tap at 3.98 fires the LZR-50 at 4.07
  and the Jax-iC, still cooling down, at 4.27, after the button came up (the one request this
  replaces went with the LZR-50's shot); the tap at 6.0 both at 6.13.
* **The scope** (`-- --test BF_NO_SQUAD=1 BF_TEST_WEAPON="L-Shot-75;Jax-iC" BF_TEST_SKILL=2,6
  BF_TEST_SCOPE=1,3`): asked for from 3.0 s, it stays out through berserk (the frames at 3.3,
  4.7 and 6.1 s from behind him) and comes in after the off event (6.67 s: in at 6.7-7.7 s).
  The hook sets the scope step every frame, so Tex stays aimed through berserk here; a real
  right-click on a zoomable gun aims for one frame only in berserk (its aim is the scope step,
  which the block clears), a demo choice until shot T2.
* **The crouch** (the taps' spot, `BF_TEST_SKILL=2,5 BF_TEST_CROUCH_KEY=3.5,6.5
  BF_MOVE_LOG=1`): Z at 3.5 s in berserk does nothing (bz_idle); after the off (5.67 s) he stands
  on; Z at 6.5 kneels (crouching at 7.07, crouched at 7.53).
* **Regression**: one gun's fire is unchanged, against main (0f9e876): `BF_MAP=flat
  BF_AUTOPILOT=1` as Tex and as Brutus (`BF_CHARACTER=0`, the Bower's walked pellets), 480
  frames, and on the test map the LZR-50's held fire, taps and dry cycles (`-- --test
  BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.535,19.5 BF_TEST_WEAPON="Confed LZR-50"
  BF_TEST_TRIGGER=1-4,5-5.08,5.5-5.58,6.5-6.58`, and with `BF_TEST_CLIP=6`), 120 frames: the
  `BF_SHOT_LOG` and `BF_AMMO_LOG` lines are main's line for line once each pellet's new `hand:
  0, ` is taken out (the only change to them). The test map's frames are main's pixel for
  pixel; the autopilot's differ only where two runs of one build differ too: the hand-over
  chevrons' phase at 31 s, and in some runs a smoke cloud's particles at 30 s.

* **The deathmatch Texes' rack draw and stow** (the product owner: the reach back to the rack at the
  press and the put-back at the off were gone for the squad deathmatch and deathmatch Tex, not the
  campaign's; `-- --test BF_CHARACTER=tex BF_NO_SQUAD=1 "BF_TEST_SQUAD_SLOT=0.3,me,tex@sdm_bf"
  BF_TEST_GOTO=0,4,0,4 BF_TEST_SKILL=3,8 BF_SKILL_LOG=1`, and `me,h_e00f4e1d` for the deathmatch
  library's Tex; 15 fps `BF_CAPTURE`, and walking: `BF_TEST_GOTO=0,4,0,-40`), against main 5482136:
  the log had the toggle and "second gun drawn" in the press's own frame (3.13 s), no "on overlay"
  line, and the off in its own (8.13 s); the frames had the gun jump into his left hand and out of it
  with the arm at his side. With the clips found through the type: the campaign Tex's lines (on
  overlay 1.00 s, switches at 0.50 s, the gun drawn at 3.67 s; off overlay, off at 8.67 s) and the
  frames of take01's draw (2.3-3.1 s into the take): the left arm up behind the shoulder (frames
  50-59), the gun lifted off the left ramp and brought down to the left hand (60-62); and at the off
  the gun up behind the shoulder and onto the ramp (124-134), standing and walking. Unchanged against main, the Tex
  region of the frames: the campaign Tex's on and off (sdm_e34 standing, the test map standing and
  walking, after a hand-over from Brutus, firing both guns: 150-190 frames each, identical but the
  muzzle flash's random edge in one firing frame), Kingman's (150 frames: his Berserker stays
  clipless) and Hawk's Stealth (150 frames); and the campaign Tex's on and off at ce07a6a, the commit
  before #407, are the same pixels too, where the sdm_bf Tex there had no Berserker (V logged and
  did nothing). `cargo test --lib skill_overlay`: ms_tex's overlays for
  the squad's, the deathmatch library's and sdm_bf's types, none for ms_kingman, Hawk's, Brutus's and
  the Feral Colonist's as the data has them.

**Guessed or open (Tex)**: the berserk walk, run, side steps and jumps aren't in any take (shot
T1); the demo moves them by their clips' roots (bz_walk 1.29 m/s against the walk's 1.62), where
the game moves at the type's speeds. The scope in berserk (shot T2), a crouch in it and the black
button (T3) aren't filmed: the demo's no scope and dropped crouch are its choices meanwhile, and
the black button has no demo key. A tap's request lasts, in the demo, until each gun's first
shot; the game's until the 0x0f8408d5 event of a clip not placed, so whether a gun still cooling
down when it comes fires at all is open. After an off with a reload following (take05), the
game's weapon list goes 1.6 s after the off, the demo's holds its 2.5 s (`HUD_LIST_TIME`): the
weapon box's timer isn't traced. Whether the game's camera pulls in when berserk fires (the
demo's does, as for every gun; take04's doesn't) is play.md's question. A toggle overlay cut
short by a knock-down (the demo's choice) leaves the motions as the skill is.

### Brutus's Spirit of Vengar (#122)

Brutus (`BF_CHARACTER=0`), `BF_CAPTURE` at 15 fps (60 for the fade), `BF_SKILL_LOG=1`, against
the takes' watch logs and frames:

* **Regeneration and drain** (take07: `BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-53.56,30.81
  BF_TEST_HEALTH=50 BF_TEST_STAMINA=30@1.9 BF_TEST_SKILL=2.0`): on at 2.07 s with 30.08, 50.53 hp
  at 2.53 s to 65.00 at 17.00 s (+1.00/s), off at 0 at 17.07 s; take07 +1.01/s, 50.0 -> 65.04.
* **Damage taken** (take27: `-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,20
  BF_TEST_NPC=0.5,brutus,enemy,fight,4.5,7 BF_TEST_WEAPON=Saryl-45 BF_TEST_INSTANT_KILL=0
  BF_TEST_SKILL=8 BF_COMBAT_LOG=1`): 12 hits off, 4.77-5.69 (mean 5.22); 12 on, each with " x
  the victim's skill 0.85", 4.09-4.87 (mean 4.50); take27 5.28 and 4.43.
* **The charge** (take08: the test-map hooks under [Keys and test hooks](keys-and-test-hooks.md),
  300 HP): on at 2.07 s, the dash from 4.0 s, the hit at 4.73 s at 1.46 m and 0 degrees: 200 x
  1.40 = 280, 300 -> 20 hp, his line said (f6627a6d), 100 stamina taken; at 4.80 s stamina 0, off,
  the view's fade; the refill x0.25 while he still dashes, then x1. take08: the hit at 1.34 m,
  280, off the tick after. With 125 HP (take22): 125 -> 0, "killed: thrown", the body thrown and
  down. Without the sprint (take23's walk; the demo's run isn't the dash) and with Vengar off
  (take24): no hit, his stamina as it was.
* **The view**, frames beside the takes': e34 at take06's spot (`BF_MAP=e34 BF_CAMERA_DISTANCE=6
  BF_TEST_GOTO=-55.78,30.28,-55.91,30.76 BF_TEST_SKILL=2.0`), m03_a at take08's tower
  (`BF_MAP=m03_a BF_TEST_GOTO=-220.74,-123.17,-185.16,-48.93 BF_TEST_GO_AT=99 BF_TEST_SKILL=2.0`:
  Tex's spawn, where the view was taken when the start read `-182.6,-43.5`, which the spawn
  overrode before #292),
  e01 at night (take09: `BF_MAP=e01 BF_TEST_GOTO=-14.8,103.66,-18.16,116.76 BF_TEST_GO_AT=99
  BF_TEST_SKILL=2.0`) and the test map with a friend and two enemies (`BF_NO_SQUAD=1
  BF_TEST_NPC="0.5,tex,friend,dummy,7,13;0.5,shrike soldier@m09_a,enemy,dummy,2,8;0.5,hawk,enemy,dummy,4,4"`):
  the grey streaked world, floors lit and walls dark, Brutus and the enemies bright, the friend
  dim, the halo, the HUD as it was. The ground's grey (the lower picture without the HUD and the
  figures, median and quartiles): e34 43 (29-56) against take06's 59 (45-75), m03_a 102 (97-108)
  against take08's 147 (138-156), e01 59 (50-67) against take09's 49 (41-72); e34's gate wall
  (13, 13, 18) against (12, 11, 19). (Round 1 drew the baked terrain light x4, fitted to e01: it
  blew e34 and m03_a out to near white.)
* **The pulse** (e01 as above, 150 frames): the figure's mean green (its pixels alone) 103-164,
  the autocorrelation's peak at 1.000 s (0.86); take09 104-175 (1.000 s, 0.82), take06 125-184
  (1.000 s, 0.81).
* **The fade** (the test map, `BF_CAPTURE_FPS=60 BF_TEST_SKILL=1.0,2.5`): green and blue x (1 -
  level), the level 1.00 in the switch's frame and 0 36 frames (0.6 s) later, falling 0.028 a
  frame, on and off; take06 G 6 -> 43 over 2.20-2.80 s and 3 -> 21 over 20.17-20.80 s. Death
  with it on (`BF_TEST_DIE=4.0`, take30): off in the frame of death and the view back from red.
* **With it off, nothing changed**: the base commit's demo and this one on the same e34 scene
  (`BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-50.69,36.29`, 40 frames) agree within 2 levels on every
  pixel (the HUD is now drawn by the heat camera and laid over the picture), but for the frames
  where loading finished a few frames apart, which two runs of either build show too.

**Guessed or open (Brutus)**: the heat figures' texture, and so their colours and the pulse's
shape (fits); the halo's weight; the vision pair's strength (`VISION_LIGHT`, a third of the code's
white: m03_a comes out darker than take08, e01 a little lighter than take09); the kill's
throw is the code's since #259 (ragdoll.md), its speed against take22's not measured (the tower
stops it); what the game's view does at a hand-over with Vengar on. On the first
switch of a session the figures are missing for two frames at 60 fps (their pipeline compiles),
hidden by the red fade then. The spec's shots B6 (the cone, an enemy 45 degrees off), B7 (a
grenade's blast on him, on and off) and B8 (a squadmate seen still at 3 and 15 m) weren't
filmed: the cone and the 0.85 on blasts are the code's.

### Hawk's Stealth (#123)

Hawk (`BF_CHARACTER=2`) on campaign e34 at the takes' range spot (`BF_MAP=e34
BF_TEST_GOTO=-52.96,30.96,-53.09,31.44`) and on the test map, `BF_SKILL_LOG`, captures at 30 fps
(15 for the logs) set beside the takes' frames at the same scale (scripts in the developer's
scratchpad):

* **take10** (`BF_TEST_SKILL=2.0,22.0`): on at 2.90, 0.83 s after the press frame; "Stealth"
  under the bars and the white bar; she fades out as one layer, no part showing through another,
  and is gone by 3.9 s; invisible against the gate to 22.87 (off), back in 1.0 s. Her presence
  (her outline's difference from the picture without her, 0 to 1) against the switch: 0.80,
  0.53, 0.38, 0.26 and 0.14 at 0.25, 0.50, 0.62, 0.75 and 0.87 s, where 1 - the level is 0.75,
  0.50, 0.38, 0.25, 0.13 and take11 measured 0.78 (at 0.24 s), 0.50 (0.49), 0.33 (0.64), 0.25
  (0.74), 0.19 (0.84, with the ripple's 0.13 under it).
* **take11** (`BF_TEST_STAMINA="12@1.9,50@8.1,9@17.9,11@19.9" BF_TEST_SKILL=2.0,8.3,9.5,14.0,18.0,20.0`):
  at 0 (7.93) off at once with no clip, the line at 0 refused (no fight); the press at 8.40,
  during the fade back, refused "mid-fade"; at 9.60 accepted; with 9 refused "under 10%", with
  11 on.
* **take13** (`BF_TEST_WEAPON="Saryl-23;Rail RVR" BF_TEST_GRENADE_TYPE=Sonic BF_TEST_SKILL=2.0
  BF_TEST_THROW=6.0,0.15 BF_TEST_TRIGGER=10.0-12.0`): the throw at 6.20, 15 off (the take's 15.1
  at 6.19-6.24), the grenade invisible in her hand and seen in flight; shots at 10.23, 10.67 and
  11.10 (the take's 10.23, 10.68, 11.09), 27 each (27.1), the third emptying the bar and the
  skill off in its frame, the burst going on; her head and shoulders back by 12.1, a Sonic's
  scorch on the ground showing through her as she comes back.
* **take14** (the test map's live Shrike, under [Keys and test hooks](keys-and-test-hooks.md),
  `BF_TEST_WEAPON=Jax-iR BF_TEST_SKILL=4.0 BF_TEST_TRIGGER=18.0-18.08 BF_TEST_INSTANT_KILL=0
  BF_SIGHT_LOG=1 BF_COMBAT_LOG=1`): it shot her until the switch (5.00), then lost her about 0.8
  s into her fade (5.87, "forgets Squad(2) (stealthed)") and landed nothing more; her shot (one
  Jax-iR round, 68.25: off at 0) showed her (18.27, a contact again), and with the 1.15 for a
  known target it kept her as she came back (before #197 it lost her at 18.53 and found her at a
  visibility of 0.27 at 18.60), and fired again (the take's went INVESTIGATE and didn't: not in,
  above).
* **The dropped gun** (#197; `-- --test BF_CHARACTER=2 BF_NO_SQUAD=1 BF_TEST_GOTO=0,4,0,4
  BF_TEST_SKILL=1.0 BF_TEST_DIE=3.5 BF_CAMERA_PITCH=-0.35 BF_SKILL_LOG=1`): she dies stealthed at
  3.53 ("dropped gun's part opaque again" for both parts); her guns are seen falling from the
  capture's first frame of the death on, where main's showed nothing of them while she faded.
* **The AI's hold fire** (`-- --test BF_TEST_GOTO=4.5,20,4.5,20 BF_TEST_STAMINA=20@1.0
  BF_TEST_SKILL=1.0 BF_TEST_SELECT=3,2.5 BF_TEST_NPC="5.0,shrike soldier@e40,enemy,fight,4.5,4.3"
  BF_TEST_NPC_HP=5.5,1,999 BF_COMBAT_LOG=1`): Hawk handed over stealthed; with the Shrike in her
  sight from 5.13 s she fired nothing while Brutus and Flint did; off at 0 at 10.13, her first hit
  at 11.00. The Shrike shot only Tex.
* **The crosshair** (`-- --test BF_CHARACTER=3 BF_TEST_SWEEP=9.6 BF_TEST_HOSTILE=hawk
  BF_TEST_SQUAD_SKILL=hawk,2.0 BF_RETICLE_LOG=2`): the hostile Hawk red, under the centre and
  locked, to 3.80 s; from 3.93 (0.94 s into her fade) blue, "hawk unseen: visibility 0.00", the
  centre still on her. Rerun for #333 (the hook through Toggle Ability's goal, run for her though
  the hook holds her): stealthed at 2.93 s, "hawk unseen" from 3.87 s, 0.94 s into her fade; but
  the test map now has her 69 units below the crosshair from the start ("hawk at (+9.8, +68.8)
  units ... outside"), so the reticle stays blue and the red phase isn't reproduced. The same
  build with the hook's old one-shot request gave the same 157 reticle lines and 75 frames to the
  pixel: the scene has drifted since this was measured, not the toggle.
* **Footsteps** (`BF_MAP=flat BF_CHARACTER=2 BF_TEST_GOTO=0,0,0,-60 BF_TEST_SKILL=0.5
  BF_SOUND_LOG=1`): her run's 0.6 down to 0.54, 0.39 and 0.21 through the fade, then 0.15 (the
  demo's own volumes then). Since #324 a step is her noise x the gait's factor (1.0 for a run) x
  0.75 (her own step, 2D): 0.67, 0.49, 0.26 through the fade, then 0.19 (a quarter of 0.75).
* **Hand-over** (`BF_TEST_SELECT=3,4.0`): on under the AI, draining 2.5/s, invisible (take19).
  **Death** (`BF_TEST_DIE=4.0`): off in the frame of death, her body back on the ground a second
  later. **The Vengar view** (`BF_MAP=flat BF_CHARACTER=0 BF_TEST_SWEEP=180
  BF_TEST_SQUAD_SKILL=hawk,1.0 BF_TEST_SKILL=3.5`): the stealthed Hawk a dim heat figure.
* **Regression**: against main (44e7780), Hawk at the e34 spot without stealth, 45 frames (her
  warm-up copies' second among them), pixel for pixel the same; take27's test-map fight (Brutus
  and an enemy Brutus with the Saryl-45) the same 19 hits, its frames the same but for the hit
  glow's particles in two.

**Her refraction (#198)**: Hawk at the same spot with a still camera (`BF_TEST_GOTO=-52.96,30.96,
-55.57,40.61 BF_TEST_GO_AT=99`: the camera faces a goal 10 m off, which she doesn't walk to; with
#123's goal 0.5 m away her idle's centimetres turned the camera by degrees), `BF_TEST_WEAPON=
"Saryl-23;Rail RVR"`, `BF_SKILL_LOG`, captures at 15 and 30 fps, against takes 01, 03 and 07 in
`todo/198-hawk-invis/` (scripts in the developer's scratchpad `t198dev/`):

* **The wobble** (take01's view: `BF_CAMERA_DISTANCE=1.8 BF_CAMERA_PITCH=-0.05
  BF_TEST_SKILL=2.0,14.0`): the gate's brace displaced inside her outline and changing shape, the
  rifle on her back a displaced band, as in take01 6-9 s. Matching every pair of frames inside her
  outline: the take's mirror-image pairs put its extremes at 4.7, 5.95, 7.25, 8.55, 9.8 and
  11.1 s and its same-phase pairs 2.4-2.6 s apart, a period of 2.56 s (the code's 2.51, at the
  take's 0.1 s steps), with the zero crossings between alike; the demo's extremes at 4.5, 5.75,
  7.0 ... s (2.51 s).
* **The strength**: block matching inside her outline against a zero-crossing frame: the take's
  shift at its extremes 16-20 px (75th percentile, its 960-wide picture), the other sign's 10 px;
  the demo's 23-28 and 13-15 px, with everything drawn 1.22-1.3x the take's size (her head 105 px
  against 81, the gate's pillar 110 against 90): 18-23 and 10-12 px on the take's scale (23 / 1.3
  to 28 / 1.22, 13 / 1.3 to 15 / 1.22; #219 corrected the 21-26 and 13 first given here), against
  the take's 16-20 and 10. The two signs differ the same way in both (the shift follows her
  surfaces' world axes). Since #269 the demo draws at the take's scale (#219: her head 80 px
  against take01's 83 at the same spot), so a new measurement needs no conversion; not redone
  (no plate of the picture without her was filmed: `todo/198-hawk-invis/shotlist.md` A).
* **take03's barrel** (the camera turned: `BF_TEST_GOTO=-52.96,30.96,-43.75,27.05`): the barrel and
  the shed's machinery behind her seen torn and doubled through her outline, as in the take at
  10-11 s.
* **The fades** (frames each 0.2 s against take01's 3.0-4.2 and 15.0-16.2 s): her skin thinning
  linearly as the tinted copy comes in, a faint ghost on the ground, gone by 4.0 s, and back the
  same way, the take's about 0.1 s later (its picture lags its log). Her presence (the projection
  of her outline's change onto her whole body's) at the default camera against main's
  (#123's take11-matched fade) at 30 fps: within 0.022 through the fade-in and 0.01 through the
  fade back: the tint and the second half's small shift are all the refraction adds there.
* **The flash** (`BF_TEST_STEALTH_FLASH=5.44,6.07` at 1.8 m and at take07's 7 m,
  `BF_CAMERA_DISTANCE=7`, 30 fps): no body; the ripple jumps in the flash's frame (at 1.8 m the
  ground's copy pulled up into her shoulders) and settles over 0.25 s. take07 (a real burst seen
  from Tex at 7 m): no body at the shots, the muzzle ball and bolt over a faint distortion.
* **The cut-short fade** (`BF_TEST_DIE=3.35`, in the fade-in): her skin whole again in the frame
  of death, the refraction dying over the second after. The code's case; not filmed.
* **The first switch-on (#219)**: the controller's update runs before the toggle clip's event
  switches it (`step`), so the switch's frame drew what the update had left: on a map's first
  switch-on the as-made alpha 1 with no offset, a full-alpha copy of her tinted (0.9, 1, 1) for one
  frame. `Cloak::switch` now works the frame's values out again with no time (switched on from
  off: alpha 0). At take01's spot at 30 fps (`BF_CAMERA_DISTANCE=1.8 BF_CAMERA_PITCH=-0.05
  BF_TEST_SKILL=2.0 BF_CAPTURE_FPS=30`), her body's mean R/G in the switch's frame (86) was 0.979
  between 1.024 and 1.026 before, a one-frame cyan cast; now 1.025, the fade starting smoothly a
  frame later. take01 shows no such frame either (her body's colour steps once at 3.04 s, then
  fades). The game's order of the event against the controllers' frame isn't traced.
* **Lamp light** (#219): play.rs's `lamp_characters` re-sent a cloaking body's materials to the
  GPU every frame once her lamp light had changed: her depth copies and twins sit under her root
  without a standard material, which it took for parts drawn with another material for now (the
  Vengar view's) and so never recorded the light as set. They're marked (`BodyCopy`) and passed
  over. Stealthed Hawk walking under sdm_e34's lamp at (-27.9, 42.8, 91.9)
  (`BF_MAP=sdm_e34 BF_CHARACTER=2 BF_TEST_GOTO=-27.9,86,-27.9,100,42.5 BF_TEST_GO_AT=2.5
  BF_TEST_SKILL=1.0`, 150 frames, counted with a temporary log): 134 re-sends before, all with
  the light unrecorded; now only the frames whose light changed (70, as she walks and sways
  under it).
* **A missing 3D render graph** now warns ("the refraction's pass isn't moved") instead of leaving
  the refraction in bevy's order silently (`RefractionOrder`).
* **Unchanged**, against main (8457fa6), pixel for pixel but for the first 2-4 frames as the map
  loads (two runs of main differ there too): Hawk with her skill off at the spot (45 frames, her
  warm-up second among them), #122's e34 scene (`BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-50.69,36.29`,
  40 frames), Tex's shots at the gate (`BF_TEST_GOTO=-53.6,30.8,-53.71,31.49 BF_TEST_FIRE=hip
  BF_CAMERA_PITCH=-0.12`, 60 frames). The Vengar view with a stealthed Hawk (`BF_MAP=flat
  BF_CHARACTER=0 BF_CAMERA_DISTANCE=4 BF_TEST_SQUAD_SKILL=hawk,1.0 BF_TEST_SKILL=3.5`): her
  refraction in the normal view, a dim heat figure in the Vengar view and nothing of the
  refraction there. take13's sequence: the Sonic in her hand given a twin and its own material
  back once thrown; its scorch seen through her outline, displaced with the ground.

**Guessed or open (Hawk)**: the enemies' goals after they lose her (H3); the vision-range as the
x3 of FUN_000481c0 (the gun's weapon-data +0x16c, not identified), left out; the sRGB alpha's 2.2 suits a dark body on a lighter picture (the demo's presence
runs up to 0.05 over 1 - the level). During a fade a see-through thing between her and the
camera (smoke, a liquid) is drawn under her skin; her shadow stays until her skin's alpha is
under 0.05 (bevy's blended shadow cut), where the game's follows the cloak's body alpha. The
refraction: its depth write (off here; the pass's state isn't traced), the shift worked out per
fragment from the interpolated normal (the game's per vertex), its blend in linear light (the
console blends stored values: the same where the copy is like the picture), the fog (the levels'
linear fog only), and the shift turned into the game's angle (`game_screen`) are the demo's;
no plate of the picture without her was filmed (the shot list's A-C), so the tint and the shift's
direction against her facing are the code's, not measured. On e34 the demo's ground is much
lighter and greener than the takes' dark dirt, so the shift shows far more on the ground inside
her outline (and the flash's ghost at 7 m) than in the game.

### Flint's Advanced Targeting (#124)

On campaign e34 at the takes' range spot, Flint with the takes' loadout and an e34 Shrike Heavy
(its sphere 1.24 m) where the takes' stood, 7 m ahead (`BF_MAP=e34 BF_CHARACTER=1 BF_NO_SQUAD=1
BF_TEST_WEAPON="L-Shot-75;Jax-iP" BF_TEST_GOTO=-52.96,30.96,-53.09,31.44 -- --test
BF_TEST_NPC="0.3,shrike heavy,enemy,dummy,-54.84,37.70" BF_TEST_NPC_HP=1.0,1,999`), with
`BF_SKILL_LOG`, `BF_TARGETING_LOG`, `BF_RETICLE_LOG`, `BF_COMBAT_LOG`, 15 fps captures set beside
the takes' frames at the same scale (scripts in the developer's scratchpad). The squad left out:
the demo's formation stands it between Flint and the Shrike. (**Not free to act**'s Frag below
was thrown before #161 made the throw the game's: its landing time doesn't hold as written. Not
run again.)

* **take15** (the L-Shot; `BF_TEST_SWEEP="0.5,1.0,24;4.0,4.5,-24;8.5,9.0,24;11.0,11.5,-24"`, the
  takes' 12-degree turns, `BF_TEST_SKILL=2,21`): on at the press; the Shrike locked at once 82
  units right of the crosshair; the corners slide in from the sides (the left one at x ~255
  0.26 s after the switch, the take's ~215 0.28 s after; the right one still on its way) and
  close round the Shrike by 0.6 s with the glyph strip (eight glyphs) over the top left and the
  wave under the bottom right, as take15 2.5-3.0 s; the box 115.5 units across (58 x 87 each
  way) with the camera in, the take's 105 (52 x 81) with the game's, which sits a little further
  back: round the body as the take's frames it (98.5 with the camera out, before the camera's
  hold); the crosshair red, then fading out by 2.6 s (the take's by 2.8); the
  rifle raised to the shoulder (from the side, `BF_VIEW_YAW=1.9`); at the off the fill falls in
  0.5 s. Drain 3.2/s.
* **take16** (the Jax-iP, `BF_START_WEAPON=1`, `BF_TEST_SKILL=2,9`): the same marker, no sniper
  motions; at the off the pistol's own lock stays on the Shrike, the crosshair red, and the corners
  close onto a 64 x 64 box at the crosshair as they fade, the strips still there: take16's
  9.17-9.55 s.
* **take17** (no enemy, `BF_TEST_STAMINA=30@1.9 BF_TEST_SKILL=2`): the wide box at (192, 96) -
  (448, 288), its bracket lines where take17's are (x 204 / 438, y 100 / 279); 30.05 to 0 in 9.33 s
  (3.22/s), off in the frame of 0 with the line at 0 and the end sound, the corners sliding out,
  the crosshair back at once, the label fading, the low-stamina blink; refill 0.8/s.
* **The pose and the camera** (take17's recipe with no NPC, without `-- --test`, set beside
  take17 aligned on the off, the demo's at 11.40 s and the take's at 11.55): far before the
  press, in a frame after it (the take's within 0.18 s of its switch, 2.09 to 2.27 s); in through
  the drain; the rifle down 4.0 s after the off (the take's 3.95-4.25); the camera in until 6.3 s
  after it and out by 6.8 (the take's out over 6.6-7.05). take29's recipe (the reload right after
  the off): the reload at 5.20 s, the rifle lowered once it's done, the camera out over 7.6-8.2
  s, as take29's 7.6-8.0.
* **take28** (single shots, the crosshair turned 7 degrees left of the Shrike, inside the
  128-unit box and off its body; `BF_TEST_INSTANT_KILL=0`; the take had the reticle on the
  body, 1 hit in 8 off and 7 in 8 on, but the demo's solid cylinders take a shot on the body with
  the skill off too, so its check aims off the body): off, 0 hits in 8; on, 8 in 8 (38-46
  on a thigh, x1.4 the STANDARD step: the take's 40-44), one doubled (80.72: the take's 80.6;
  `BF_TEST_DOUBLE=100` doubles every one); the 12th round ended it in its frame (take28 51.1 s).
  Scoped (`BF_TEST_AIM=1`, standing): x1.5 on every roll (29.3 on a calf, the take's 29.6 "leg";
  34.0, 28.7, 33.4); in the scope the lock holds and the corners sit at the safe area's corners,
  the glyph strip top left and the wave bottom right, as take15's scoped frames show. A doubled
  roll's frame draws the corners dark red, the strips as they are.
* **take29** (`BF_TEST_CLIP=1`, a shot at 5 s): the clip empty and the skill off in the shot's
  frame (5.13), the reload in the next (12 rounds, the reserve 50 to 38).
* **take31** (`BF_TEST_CLIP=8 BF_TEST_SKILL=2,6,10 BF_TEST_SWITCH=4 BF_TEST_RELOAD=8,12`): Q ends it
  with no switch; R ends it (8.13) and the reload follows in the next frame (8.20, 4 rounds); R
  with the clip full does nothing.
* **Not free to act** (`BF_MAP=flat BF_CHARACTER=1 BF_TEST_GOTO=0,4,0,4 BF_TEST_GRENADE_TYPE=Frag
  BF_TEST_SKILL=1.0,4.0 BF_TEST_THROW=2.0,0.3,5.5,0.3 BF_GRENADE_LOG=1`): the throw at 2.0 s, on,
  does nothing; the one at 5.5 s, off since 4.13, lands at 6.60. The medkit (e34 on a Medkit
  pickup, `BF_TEST_GOTO=-59.6,0.9,-59.6,0.9 BF_TEST_HEALTH=50 BF_TEST_MEDKIT=3.0
  BF_PICKUP_LOG=1`): used with the skill off (50 to 90), not with it on from 1.0 s.
* **Refusals**: the clip and the reserve empty (`BF_TEST_CLIP=0 BF_TEST_RESERVE=0`): "the clip in
  hand is empty"; in the autopilot's running jump (`BF_AUTOPILOT=1 BF_TEST_SKILL=17.25,18.5`): "no
  motion type 45 transition", then on at 18.6 on the ground, the run becoming the sniper walk.
  With a Frag in the hand (`BF_MAP=flat BF_CHARACTER=1 BF_TEST_GOTO=0,4,0,4
  BF_TEST_GRENADE_TYPE=Frag BF_TEST_THROW=2.0,1.0 BF_TEST_SKILL=2.5,3.5,5.0`): refused charging
  (2.60 s) and throwing (3.60), on at 5.13 after the throw.
* **The walk** (`BF_MAP=flat BF_TEST_GOTO=0,0,0,-60 BF_TEST_SPRINT=0.5 BF_TEST_SKILL=3,6
  BF_MOVE_LOG=1`): the sprint becomes the sniper walk at the press and comes back at the off.
  Without the sprint (`BF_CHARACTER=1`, no `BF_TEST_SPRINT`): the run becomes the walk at 3.13 s
  and the run is back from the off at 6.13 (the autopilot doesn't set the walk input each frame,
  so the cap stays on the frame's gait, not on the input).
* **Hand-over** (`BF_TEST_SELECT=3,4.0`, the squad in): the AI's Flint stays on and drains 3.2/s;
  Tex's HUD has no marker and a full crosshair.
* **The aim at the lock, for everyone** (Tex on the test map, the LZR-50, an enemy NPC 9 m out;
  `-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,19.5 BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC="0.5,tex,enemy,dummy,4.5,11" BF_TEST_SWEEP="1.5;4.5,4.6,60"
  BF_TEST_TRIGGER="2-3,6-7"`): the burst with the crosshair red (inside the LZR-50's box) hits 4
  of 4; the one 49 units off (blue) misses 4 of 4.

**Guessed or open (Flint)**: what makes the game's rifle lose its lock for 1.0-1.4 s at a time
(take15: tight, wide, tight with the camera still) and the pistol lock only once the crosshair
came near the enemy (take16 2.5-4.5 s): the demo locks whatever is in the box and in sight, with
either gun (shot F1). The game's L-Shot also had no lock after take15's off where the demo's
tiny-box lock caught the Shrike, so the demo's corners closed on the crosshair there. The sphere
leaves out the bones' own spheres (FUN_00236670's union of the posed pieces). The game's corners
move whole units a frame, the demo's by the frame's time. The 0.65 run threshold stands for the
demo's walk key; the sniper motions' speeds are their clips' roots. The black button has no demo
key. The camera comes in by the game's 1.75 m along the crosshair ray since #126
([camera.md](../camera.md); before, the demo's aiming dolly framed her less tightly than the takes').
The reload ending the pose and the camera's hold is a fit to take29 (the code that clears
the ready timer there isn't found). The box's size against the take's waits on the cameras
matching: re-measured after #129's merge with the camera in, 115.5 units to the take's 105, where
it was 98.5 with the camera out; the sphere leaves out the bones' parts (above).
