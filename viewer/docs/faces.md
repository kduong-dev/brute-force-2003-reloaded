# Faces (`bf/face.rs`, `bf/audio.rs`, `src/bin/play_faces.rs`, #255)

How a character's face is drawn in play: the game's face controller, not a clip. #255's first
slice made **the face table, faces at rest and the blink**; the second (#325) **the lip-sync**:
a speaking character's mouth follows its line's viseme tracks from the sound library, on every
speech path with a speaker. The expressions come in their own slice (#326): their fade runs, but
nothing sets one yet. Findings behind it: the code map's "Faces" section (`code-map.md`); the
analyst's leads are `todo/255-facial-animation/leads.md`.

## What the game does

* **No face clip plays while a character talks.** Each character with a face (`<face
  viseme-file>` in its type) has a face controller (char+0x5b8, FUN_00143380), made with the
  character. Every tick it blends 22 static poses, weighted, over a base pose and writes the
  result into the face bones' joints (FUN_001439b0, FUN_00143820). The poses are read once per
  type from the `<name>face` animation set (FUN_00142df0, the table at type+0x1ac).
* **The face bones are the controller's.** FUN_00143380 flags each one's node (+0x44 bit 5), and the
  animation binder (FUN_00234190) skips flagged nodes: no body clip ever moves a face bone.
  So the faces, modelled mid-speech, show the controller's pose and nothing else.
* **At rest** the face is `Sc_base`, the base pose: the controller writes it as it is made
  (FUN_00143380's last call). While nothing shows (no line speaking, no expression) the controller
  writes nothing, so the bones keep the last pose they were given.
* **The blink** (FUN_00143d10) is the one face clip that plays: `Sc_blink`, 0.333 s, the lids.
* **The lip-sync** is in the sound library, not the audio: every speech sound carries a curve per
  mouth shape, and while a character says it the controller writes the curves at the time since
  the line began into the weights (below, "Lip-sync").

## The data

The game's 10 face sets (`brutusface`, `flintface`, `hawkface`, `texface` in common; `militiaface`,
`mutantface`, `seerface`, `shadoon_face`, `shadoon_head_face`, `kingmanface` in the levels' and the
multiplayer archives, 39 copies in all; `viseme-file=""` means none) each have the same 24 clips (Flint's has
one more that nothing looks up):

| clip | what |
|---|---|
| `Sc_<name>` x 22 | the 22 visemes of default.xbe's VE_ table (0x3be090): 15 mouth shapes (`bump cage church earth eat fave if new oat ox roar size though told wet`, the Microsoft SAPI viseme set) and 7 expressions (`angry base confused great_pain happy sad surprised`). Each is a still pose: two keys over 0.033 s, or one pose held 1 s for an expression; the game reads the first key (index 0 is pushed at 0x142ed9 and 0x14314b) |
| `Sc_base` | duration 0, one key: the rest pose, and by its targets in order the **face bone list** |
| `Sc_blink` | 0.333 s, the lids' rotation keys |
| `Sc_rest` | a display object's, not used in play |

The face bone list is what varies, by the model: 1 bone for the creatures and the soldiers (the
jaw, one rotation channel); 18 for Flint, Tex and the Shadoon (3-4 rotations and 14-15 positions:
brows, cheeks, lips, nose); 22 for Hawk (four more rotations, her lids). A bone with a quaternion
channel is a **rotation bone**, one with a vector channel a **translation bone**.

Of the game's 128 character types 89 have a face (`BF_DUMP_TYPES=all` lists each model's table):
45 with a 1-bone list and 2 lids, blinking every 4 s; 18 Seers (3 s); 2 Kingman (5 s); 10 with a
1-bone list whose 43-bone model has no lid bones (the Red Hand, Terrorist and Militia); 11 with 18
bones and 4 lids (Flint, Tex and the Shadoon, 4 s); 3 with 22 bones and no lid that blinks (Hawk and
her two multiplayer copies).

The type's `<face>` gives the set, the blink interval (h_09109ae2, an int in seconds) and chance
(h_f5a7b9d2), and a list of `<h_fa110e09 enum scale>` that stretch a viseme (`enum` the VE_ index):
Flint's `if` x 1.5, Tex's `if` x 3, Hawk's nine mouth shapes x 1.5.

## The face table (`bf/face.rs`)

`FaceTable::build` (FUN_00142cb0 and FUN_00142df0) is made with each model by `Character::load_type`
(and `load`): `Character::face`, None for a model without a set. It holds

* `bones`: `Sc_base`'s targets the model has, each with its kind (a channel the game's code takes
  is a quaternion or a vector one, the only ones in the data) and its place in the skeleton;
* `base`: their values at the first key, 4 floats each (x y z w; a position has w 0);
* `visemes`: 22 records, each a copy of the base overwritten by what the clip `Sc_<name>` names:
  a rotation bone takes the clip's quaternion, a translation bone **the clip's position less the
  base's** (a difference to add). A viseme with no clip is the base. A record's `scale` is 1, or
  the type's stretch;
* `blink`: `Sc_blink`'s duration and the (bone, channel) of each target the model has that **isn't
  a face bone** (see Hawk, below), and the type's interval and chance.

`FaceTable::blend(weights)` is FUN_001439b0's loop: the base pose, then each viseme with a weight
over 0 in the VE_ order (mouth shapes, then expressions): w = min(weight, 1) x scale; each face bone
whose viseme value isn't already the running one (FUN_00144e50, within 1e-5 per component) takes it: a
rotation bone **nlerps** from the running value toward the viseme's quaternion by w (FUN_00233b10:
the short way round, normalised, w not clamped, so a stretched viseme overshoots its pose), a
translation bone adds the viseme's difference x w (FUN_00144e20). The mouth is a chain of blends of
absolute poses in a fixed order, not a sum of offsets; a weight of 1 on one viseme shows its clip
exactly. `FaceTable::write` puts a pose on a skeleton's local transforms (a quaternion joint takes
the rotation, a vector joint the position: FUN_00143820 and FUN_00234dc0).

`FaceParams` (`CharacterType::face`, parsed by `character_type.rs` `face_params`) carries the
type's interval, chance and stretches. `bf/face.rs` also has the viseme names (`VISEME_NAMES`:
the clip is `Sc_` + the name; `clip_name` gives a clip's name for the viewer; `viseme_by_hash` a
sound track's viseme), the blink's ease (`blink_weight`; `write_blink` eased, `write_blink_clip`
the bare clip), and the head bone's hash and the height of a face over it for the close-up cameras.

## The controller in the demo (`play_faces.rs`)

`Player::facial` (`faces::Face`) is the controller: the 22 **weights** (the line's tracks write
0-14, the expression fade 15-21), the face bones' output pose, its clock (+0x40), the line it says
(its tracks, start and time left: +4, +0x44, +0x50, +0x54), the expression's target and shown one
(+0x58, +0x5c) and the blink's clock. It is made as the character is (`spawn_unit`,
`faces::spawned`) and ticked by `step_player` (`faces::step`) after everything that poses the
body: the clip layers, the weapon switch, the reload, use, throw and skill overlays and the aim's
turn. No body pose carries a face clip any more (`Character::pose` is called with no face: the
game's body clips don't move these bones), so the face and the body never fight: the
controller's pose is written over the bones last.

* **Tick** (`Face::tick`, FUN_001437d0): no time, nothing. The clock goes on by dt, a line's time
  left down (at 0 or under it stops speaking), then the blend (`Face::blend`, FUN_001439b0): the
  expression fade (`Face::fade`, FUN_001438d0) says whether an expression shows; **with no line
  speaking and none showing, the line is stopped (the mouth's weights zeroed) and nothing is
  written**, so the face keeps its last pose; speaking a sound with no speech block and no
  expression, nothing either; otherwise the base pose, the line's tracks at the time since it
  began, and the blend. Then the blink.
* **At rest** the face is `Sc_base` on the face bones from the first frame.
* **In the world only**: a squad member a spawn-trigger hasn't spawned yet (`Player::absent`) isn't
  ticked, though its step runs (the controlled one's does, for the game's clock): the game's
  controller comes with the copy the trigger makes, so the blink's phase is the arrival (below).
* **Alive only**: `step_player` runs for a living character, as the game ticks the controller from
  the character's update (0x11dc7b: bit 0 of char+0x70 clear). A dead character's ragdoll starts
  from the pose it last stood in (`last_world`), so its face stays as it died; a knocked-down one
  that is still alive isn't ticked in the demo (the game's would blink on, a gap).
* **Cutscenes**: a held character is stepped like any other, so Tex in the second patrol's close
  shot blinks on its schedule.

### The blink

FUN_00143d10: a clock (+0x4c) of the time since the last blink, **starting at 10 s** (FUN_00143380),
counts up each tick; once it passes the clip's length (0.333 s) and **the type's interval** (h_09109ae2:
4 s for the squad), `Sc_blink` plays once and the clock is zeroed (not reduced by 4: the overshoot
of up to a tick is lost, so a cycle is 4.0 s and up to a tick). So the first blink is at the
character's first tick, then one every interval plus up to a frame: at 15 fps (BF_CAPTURE) 4.07 s,
in real time 4.0 s. Between the clip's length and the interval the game tests a random number against
the chance (0.05), but its number runs 0 to 131072 (never scaled to 0-1, then divided by dt x 30),
so the test passes about once in 2.6 million frames: the demo doesn't have it, and the blink is a
regular cycle. **The footage confirms both** (todo/255 take01-03, the face log of all four squad
members, 24 resets): every controller's +0x4c counts up and resets to 0 every 4.0 s of game time, with
no scatter and no early blink. The gaps are 3.97-3.99 s on average in the log's own clock, which runs
1-2 % short, and 4.01-4.03 s in the game's float clock: the 4 s and the overshoot of a tick or so.
On screen Tex's eyes close about 0.1 s after a reset (logged at video 4.76 s; closing at 4.88, shut
4.95-5.15, open by 5.20 s): 0.3 s, the eyes only.

**The phase is when the character was made.** The clock starts at 10 s, so the first blink is the
first tick, and the cycle runs on from there: two characters blink together only if they were made
and first ticked together. The takes' four are not in lockstep: the same period, with fixed offsets.
Their controller clocks (+0x40, which counts from the controller's first tick) lag the session's
float clock by 9.16 s (Flint), 9.63 (Brutus), 10.14 (Hawk) and 10.64 (Tex): the four controllers
were made 0.5 s apart in that order, and their blinks come 1.48, 1.01 and 0.51 s before Tex's in
the four-second cycle (2.50, 3.02 and 3.47 s after it, as measured from the resets). In each
controller's own clock the resets fall at the same places, within 0.2 s, 45 cycles in (0.8-1.0 s past
a multiple of 4: those cycles' overshoot, about 0.017 s each, from a first blink at the first tick).
The demo matches it by **not ticking a character that isn't in the world yet**: a squad member a
spawn-trigger spawns is `absent` until its copy arrives, and the controlled one's step runs all the
same (for the game's clock), which ticked its face from the first frame (Tex blinked at 0.10 s on e34
before he had arrived). Now the first blink is the arrival: e34's Flint 9.00, Brutus 9.50, Hawk
10.00 and Tex 10.50 s into the script (its timers: Flint's spawn at 9.0 s, the others 0.5, 1.0 and
1.5 s after it, the footage's spacing; its session clock reads 0.15 s more), the tutorial's Tex
17.09 s with the farcast. Characters a level makes together (the test map's squad, its placed
enemies) blink together: by the same code, no take caught it.

The clip plays on its own bones: the lids, which aren't in `Sc_base`'s list, so the controller doesn't
own them. After the 0.333 s the nodes keep the clip's last key (a held pose, what the game's nodes
do when nothing writes them again; inferred: no take could read it, Tex's lids rest where the bind pose
has them and Brutus's can't be read through his brow). Before the first blink the lids are the model's
own (the bind pose) and the first tick ends that.

**The clip is eased in** (#325). FUN_00143d10 sets a 0.1 in the play request's +0x10, and the node
controller reads it (0x2536f0): it builds a **transition**, the game's own word ("transition, ease =
..., t = ...", the string at 0x404df8), from the node's pose to `Sc_blink`'s channel, the channel's
weight smoothstep(t / 0.1) (0x280330: t / 0.1, 1.0 from 0.1 s on; the request's bit 2 asks for the
smoothstep, 0x39a848 = 3.0). So it is a **fade-in over 0.1 s, not a start delay**: the clip's own
time runs from 0, the lids are at its value from 0.1 s on (the peak, 44.5 degrees for Tex's big lids,
doesn't move) and before that they lag it: for Tex's big lid 6 % of the peak at 1/30 s and 46 % at
2/30 s against the bare clip's 23 % and 62 %, so 10 % shut comes 0.025 s later, 20 % 0.020 s, half
0.013 s, 80 % 0.004 s (worked from the clip's keys, linear in angle between them). The demo does it
in `FaceTable::write_blink` (`face::blink_weight`): each lid from where it was as the blink began, the
model's own lid at the first blink (the transition's "pose" source, FUN_00280100: nothing else moves
the lids) and the clip's last key at the others (where the last one left it), blended toward the
clip's value by the weight (the transition hands the blend to the channel's own, 0x234bf0 -> its
vtable +0x24, whose maths isn't read: a normalised lerp, as a channel's keys are blended, inferred).
`BF_FACE_CHECK` prints a first blink's lids at 1/30 and 2/30 s eased and bare: Tex's big lids 2.7 and
20.4 degrees against 10.4 and 27.4 (6 % and 46 % of the 44.5 against 23 % and 62 %, as worked out
above). That closes 0.02 s of the tester's 0.07 s against take01; the rest is not explained by the
code (guesses: the log samples at 15 Hz and sees a reset up to 0.067 s late, and a frame or two
between the controller's tick and the animation's update, none measured).

**Hawk doesn't blink.** The face controller flags every face bone's node, the binder skips flagged
nodes, and Hawk's `Sc_base` lists her four lid bones (the 22 bones of her list: four more than Flint's
and Tex's). So by the code `Sc_blink` finds no node to bind on her: it plays, and nothing moves. (Her
`Sc_base` holds her lids' rest values, equal to the blink's first key, so they sit right.) The demo
leaves those targets out and so shows none. **Take03 confirms it**: her +0x4c resets every 4.0 s like
the others', and the frames at the resets (4.02 and 8.24 s) show her open eye unchanged (seen in
profile and crouching, the first two blinks only).

How far the lids' rest keys lie from the bind pose: none for Flint and Tex (0.0 degrees), 20 and
7.5 degrees for Brutus (his eyes narrow when the first blink ends). Take02 couldn't read it: his eyes
are red slits under a heavy brow, and his head idles, so no lid closing and no narrowing between his
first blink and the later ones could be seen. Left as it is.

## Lip-sync (#325)

### The data: viseme tracks in the sound library

Every speech `<Sound>` (Type h_012ed199) in `sounds-<level>.xml` and `sounds-common.xml` has a
speech block of the same name: `<h_012ed199>` -> `<h_0128a038>` -> one `<h_f3280a84>` track per mouth
shape, named by its viseme's hash (`h_fcf7be63`: `if` h_e1d3067c, `size` h_0162b262), of `<frame
time weight>` keys (seconds into the line, which may be negative; percent). The 61 sound files hold
13294 speech sounds and 96607 tracks (scanned): every speech sound has an `if` track, the tracks name
only the 15 mouth shapes, each has 3 to 273 keys in time order with no two at one time, and the
weights peak at 85 or so, at most 140. The `h_f63f0f49` (30) and `h_11b737e0` beside them aren't read
by the face code.

`bf/audio.rs` reads them with the library (`SoundBank::add` -> `Visemes::read`; `SoundBank::visemes`
gives a sound's, shared): the runtime block FUN_0017b800 builds into the library entry's +0x1c, a
slot for each of the 22 visemes filled by the tracks' names (`face::viseme_by_hash`: their places in
the hash table 0x3c0290; a track naming another is dropped, a later track of one viseme takes its
slot). A key is made {time, weight, 0, 1.0}, and FUN_0017a990 then sets its last field to 1 / (the
next key's time - its own) on a track of more than two keys (`VisemeKey::inv`): a track of two would
keep 1.0, so its fraction would be the bare seconds since its first key (none in the data has two).

**The curve** (`VisemeTrack::sample`, FUN_0017aa70): before the first key its weight, after the last
the last's, else the key whose span holds t and the next, mixed by (t - its time) x its `inv`; x 0.01.
`Visemes::write` (FUN_0017aa30) puts each track's value into its viseme's weight; the visemes no track
names keep theirs. **Checked against the game's own output**: take07's face log (Tex's "yeah yeah",
h_ec50fb88) and take04's (his skill-on line h_05410be5 and his order answer h_0a4918dc) read the
weights at the controller's clock less the line's start; the rule gives every logged mouth weight to
the log's three decimals (49, 27 and 10 rows, the largest difference 0.006, the rounding), but each
line's first row, read between the speak and its first tick (the weights still zeroed).

### Saying a line (the controller)

* **Speak** (`Face::speak`, FUN_001435d0's part on the controller): it stops what it was saying
  (`Face::stop`), then, if the library has the sound (FUN_0017beb0), keeps its tracks (+4), speaks
  (+0x54) for the sound's length (+0x50, `SoundBank::length`: 3.047 s for h_ec50fb88, the take's
  `left` at its start) and notes the clock (+0x44 = +0x40). A sound without a speech block speaks
  without tracks: the face isn't moved. The voice itself is the caller's.
* **Each tick** the clock goes on and the time left down; while the time left is over 0 the blend
  writes the tracks at the time since the line began (`+0x40 - +0x44`) and blends the poses. So the
  mouth is the line's own curve, nothing read from the audio, and it moves in fixed steps of the
  frame.
* **The line's end**: the tick that takes the time left to 0 or under ends it; the blend then stops
  (the mouth's weights zeroed) and **writes nothing**, so the face keeps the pose of the line's last
  frame. A line that ends mid-word leaves the mouth there: take07's ends with `if` at 0.44 (Tex's
  `if` is stretched x3) and its last frames show the mouth still open after the line; the demo's
  does the same.
* **Stop** (`Face::stop`, FUN_00143570): the line dropped and the mouth's weights zeroed, the face
  not written: a chatter cut-off (0x13fbde) or a dialog-trigger's STOP (FUN_001267f0).

### The speech paths (`faces::speak` and `faces::stop`)

| Path | The game | The demo |
|---|---|---|
| chatter: every squad and enemy line (hurt, death, kill, follow, Move To, medkit, refused gun, skill, charge) | the sound event's start FUN_0013fab0 -> FUN_001435d0 (0x13fb31); its stop 0x13fb90 -> FUN_00143570 (0x13fbde) when the controller is speaking | `Chatter::voice_start` / `voice_stop` (play_chatter.rs) |
| a dialog-trigger's line (a cutscene's on-camera lines too) | 0x206d20 -> FUN_001267b0 -> FUN_001435d0 (0x1267d1); STOP -> FUN_001267f0 -> FUN_00143570 | `Triggers::dialog` queues it for `speakers` (play_sound_triggers.rs) |
| the world's dialog queue (the radio: Bratton's lines, the radar's cues) | FUN_00179ec0 plays a 2D voice with no character | nothing: no face moves |

A character without a face table (a Shrike, a hound) has no controller: its lines move nothing.
A dead speaker's controller takes the speak (a death cry) but isn't ticked any more, as the game's
isn't (0x11dc7b).

**When the mouth starts.** The two paths sit at different places of the game's world update
(FUN_000d9a20): the chatter manager's update (FUN_00140d70 at 0xd9ac7) runs before the objects'
ticks (FUN_000d9840 at 0xd9d0b); the cutscene scripts' update (FUN_00174270 at 0xd9d18, whose 103
calls the dialog-trigger's handler there, 0x1772b0) and the triggers' loop (FUN_000d99d0 at 0xd9d20)
after them. So a chatter line's mouth moves in the frame its sound starts (its first tick reads it
at dt), and a dialog line's from the next frame (its sound starts after the frame's tick): take07's
log caught Tex's controller between the two, the line begun (+0x44 = +0x40) and its time left whole
(3.047 s), while take04's chatter rows are read at the chatter's breakpoints. (A dialog-trigger
signalled from before the ticks, by the timed messages the world drains at 0xd9a94, FUN_000e40d0,
if any reach one, would move its mouth in the first frame too: not traced, and the demo doesn't
tell it apart.)

In the demo both run after the characters' step: the chatter's `update`, and `speakers`, which
puts the triggers' lines on the faces after the step and after the chatter, as the game's trigger
loop comes after the chatter's manager (a dialog line said in the same frame as a chatter sound on
the same character wins). The dialog line starts as the game's; the chatter's speak is put back by
the frame's dt (`ago`: its start and its time left), so from its next frame on its mouth reads the
curve where the game's does and ends where the game's does; only the line's first frame differs,
the game's showing the curve at dt and the demo's the face from before the line (a line can start
mid-shape: Tex's skill-on line h_05410be5 has `size` at 0.77 at its start).

The demo's chatter keeps its own copy of the time left (`Player::speaking`, the HUD's speech icon and
the priority test of a new sound, play_chatter.rs), run down by its own update; the controller's is
the face's. A dialog line doesn't take its character's chatter voice in the demo (chatter.md, "Not
done"): it plays as the triggers' voice, and the chatter doesn't see it.

### The cost

`BF_FACE_COST=1` (the test map, `BF_TEST_FULL_SQUAD=1`, the four saying a line every 1.5 s with
`BF_TEST_SPEAK`, 15 fps): about 11-12 microseconds a frame for the four faces' steps while they speak
(2.8-3.0 a face), against 3-6 at rest: nothing next to the frame.

## bf_viewer

`Character::default_face` is `Sc_base` now (it was `Sc_bump`, the b/m/p mouth). The viewer's face
(`Q` / `E` cycles, `BF_FACE=game|none|<n>`) starts as **the game's rest face** (`FaceTable::write_rest`:
`Sc_base` on the face bones and the blink clip's last key on the lids: what a controller shows once
its first blink is over), then the raw bind face, then each clip of the set with its name (`Sc_roar`;
the one clip the game never looks up shows as h_febf7c82). `BF_CLOSEUP=1` keeps the camera on the head
bone as it is posed (Brutus is hunched: his head hangs lower and forward of where the others' are);
`BF_CAMERA_DISTANCE`, `BF_CAMERA_YAW` and `BF_CAMERA_PITCH` frame it (Brutus, whose face looks
down, wants more of the first and a pitch; the demo's `BF_TEST_FACE_CAM` frames his at `1.1,-0.3`).

The ticket's "bf_viewer draws `Sc_base` blank" was not a sampling bug: a one-key clip samples fine
(`sample` took its interval from the channel, 1/30 s). The blank frames were the head-height
close-up aimed at Brutus's hunched head with `BF_CLOSEUP` (the same frame is blank for `Sc_bump`).
`sample` still guards a channel interval of 0, which none in the data has.

## Test hooks

* `BF_FACE_LOG=1`: each face as it's made (its bones by kind, the blink's lids and what it leaves out,
  the chance, the stretches), each blink as it starts (`t=4.07 tex: blink 2`), each line said or
  stopped and by which path (`t=3.27 tex: says h_ec50fb88 (3.047 s, 11 viseme tracks; a
  dialog-trigger)`), and while a line speaks, its weights each tick (`t=3.33 tex: h_ec50fb88 at 0.0667
  s, 2.980 s left: if=0.178`: the time since its start and the time left, the controller's +0x40 -
  +0x44 and +0x50, as the takes' face logs read them), then its end (`h_ec50fb88 said`); with
  `BF_TEST_FACE` each weight set.
* `BF_FACE_CHECK=1`: for each face made, how closely its table reproduces the clips it came from: the
  base and the 22 poses (each viseme at a weight of 1 over its scale) against `Character::pose` with
  the clip laid over the same body pose (the viewer's way), on the bones the clip names: the largest
  turn and move; the lids' first and last key against the bind pose; and a first blink's lids at 1/30
  and 2/30 s, eased in and bare.
* `BF_FACE_COST=1`: each second, the faces' steps' time a frame (the controller's tick and the pose
  written), the steps a frame and how many spoke.
* `BF_TEST_SPEAK=<s>,<who>,<sound>[;...]` (play_chatter.rs): at `s` seconds `who` says the sound (its
  hex id) on its voice, as a chatter line's sound starts but with no gate: its mouth follows the line.
  `--test BF_TEST_SPEAK=1.0,me,05410be5 BF_TEST_FACE_CAM=0.5,me,0.7` is Tex's skill-on line close up.
* `BF_TEST_FACE=<s>,<who>,<viseme>,<weight>[;...]`: at `s` seconds (the controlled character's clock) set a
  viseme's weight on `who` (`me` the controlled one, a squad name or 0-3, `all`) in its **test pose**
  (`Face::test`, not the game's): while one of its weights is over 0 the blend shows the test pose in
  place of the controller's weights (a line said meanwhile runs its time but doesn't show). It stays
  until another entry changes it. **With every weight back at 0 the face keeps the last pose
  blended** (the game's idle controller writes nothing, so the bones are left as they were). To go
  back to the rest face set the `base` viseme to 1: it blends to the base pose and no further, so the
  face is `Sc_base` again. `--test BF_CHARACTER=3 BF_TEST_FACE="1.0,me,roar,1;1.4,me,roar,0;1.4,me,angry,1;2.0,me,angry,0;2.0,me,base,1"`
  shows the roar, the snarl, then the rest face.
* `BF_TEST_FACE_CAM=<s>,<who>[,<m>[,<rise>]][;...]`: at `s` seconds the free camera goes `m` metres (0.8)
  in front of `who`'s face (`rise` m higher: Brutus's hangs low, `1.1,-0.3`) and follows the head as the
  idle turns it. Test sessions only (`--test`), where the free camera is.
* `BF_DUMP_TYPES=all` lists each type's face table beside its model; `BF_DUMP_SOUNDS` prints the
  character's.

## How it was checked

`--test BF_FACE_CHECK=1 BF_TEST_FULL_SQUAD=1` for the four, at 15 fps captures with `BF_NO_HUD=1` and
the cameras above, against bf_viewer's renders of the same clips:

* **The table reproduces its clips**: the base and all 22 poses of Tex, Brutus, Flint and Hawk, by
  their weights of 1, against the clips laid on the same body pose: 0.0000 degrees and 0.000000 m.
* **Faces at rest** (`BF_TEST_FACE_CAM=0.1,me,0.7`, the four, each alone on the test map: Brutus's
  at `1.1,-0.3`): a closed mouth and the brows of `Sc_base`, no mouth mid-word (the faces wore
  `Sc_bump` before, lips pressed, and the other bones at the bind pose). The viewer's `Sc_base`
  shows the same on every one; the lighting and the body pose differ.
* **The blend** (Tex, `BF_TEST_FACE`): `roar`, `angry`, `surprised` at a weight of 1 against the
  viewer's `Sc_roar`, `Sc_angry`, `Sc_surprised`: the parted lips, the bared teeth and knitted brows,
  the open mouth and raised brows; `bump` and the base after.
* **The blink**: a log of four blinks at t = 0.07, 4.13, 8.20, 12.27 s on Tex alone (15 fps; each
  4.07 s apart), and at 0.07 s then every 4.0-4.01 s in real time over 32 s; the frames show the lids
  down and up over three frames (0.2 s) at the first and the second. A character spawned later (the
  tutorial's Red Hand, which has no lids to show it) has its own first blink at its first tick (the
  log: t=6.93 for one the trigger made at 6.9 s).
* **The phases** (round 2, against take01-03's logs: `python phase.py` over their `watch.csv`, the
  controllers' clocks against the session's, the resets): `BF_MAP=e34 BF_FACE_LOG=1`, in real time over
  45 s. The first blinks: Flint 9.00, Brutus 9.50, Hawk 10.00, Tex 10.50 s, then each every 4.0 s (Flint
  13.00, 17.00, 21.01 ...); the footage's controllers started 9.16, 9.63, 10.14 and 10.64 s into its
  session, 0.47, 0.50 and 0.505 s apart. Before the fix Tex blinked at 0.10 s, with the others.
  The tutorial's opening: Tex's first blink at 17.09 s (his farcast; the Red Hand placed at the
  start blink from 0.10 s). The test map's four, made together, blink together at 0.07 s.
* **A cutscene** (`BF_MAP=tutorial BF_TEST_GOTO=34.5,75,34.5,67.5`, `tutorial_show_patrol_2`'s close
  shot of the held Tex): the face intact, the blink at 4.13 s on the camera 5 m away.
* **A death** (`BF_TEST_DIE=1.0`, from above): the face as it died, no stretched geometry, the head
  thrown back.
* **The lip-sync** (#325, 15 fps captures, `BF_MUTE=1`), the weights logged with `BF_FACE_LOG` and
  compared with the tracks (the rule checked against the takes' logs, "Lip-sync" above):
  * **a dialog-trigger's line** (`BF_MAP=tutorial BF_TEST_GOTO=37,67.5,37,67.5,32
    "BF_TEST_PANEL=a,1;a,1;a,1;a,1"`: Tex on the plate starts `tutorial_show_patrol_2`, whose 103
    fires `trigger_dialog_tex_yeahyeah`): Tex says h_ec50fb88 (3.047 s, 11 tracks) as the trigger
    fires; the first tick reads it at 0.0667 s (the frame after, as take07's controller), the last at
    3.000 s with 0.047 s left; its 45 ticks' weights are the tracks' to 0.0009 (the log's rounding).
    On the script's camera (Tex a few metres off) the mouth is shut before the line, opens and closes through
    it, wide at the `if` peaks, and is left half open after it (`if` 0.61 and `told` 0.13 at its last
    frame). Take07, on the same camera, shows the same in profile: the mouth open at the `if` peaks
    (video 10.47-10.67 s) and still half open after the line (13.0 s). Its Tex turns to the camera
    during the line, the demo's doesn't (the cutscene's body, not the face);
  * **chatter lines** (`--test BF_TEST_SPEAK=1.0,me,05410be5 BF_TEST_HURT=tex,10,3.5
    BF_CHATTER_CHANCE=100 BF_TEST_FACE_CAM=0.5,me,0.7`): Tex's skill-on line on his voice (1.596 s,
    take04's `left`; 8 tracks) is first read at 0.1333 s (the chatter's speak put back a frame: the
    game's first frame reads it at dt, its second at 2 dt) and last at 1.533 s with 0.063 s left, its
    23 ticks' weights the tracks' to 0.0007; close up the mouth opens wide on `if` (Tex's x3), works
    through the line and stays half open after it (`if` 0.31, `told` 0.43). Then the hit's grunt, asked
    through the gate (e856009f, variant 1f131100, 0.584 s, one track): the mouth opens and closes
    again over 0.6 s;
  * **the radio moves no face** (`--test BF_MAP=tutorial BF_SCRIPT=off
    BF_TRIGGER_FIRE=trigger_sound_bratton_baseisguarded,USER1,1.0 BF_TEST_FACE_CAM=0.5,me,0.7`):
    Bratton's line (h_e3656325, 1.97 s, a sound-trigger's, heard alike everywhere) plays from 1.00 to
    2.93 s; no face says anything (the log), and Tex's face close up keeps its mouth shut through it.
    The patrol-2 run has the dialog queue's Bratton line (h_081fd9f8 from 0.07 s) before Tex's own,
    with no face moving either;
  * **the cost**: above;
  * **the blink's ease**: `BF_FACE_CHECK`'s numbers (§ The blink); the second blink of the chatter
    run shows the lids half down a frame in, shut the next, and open again by the fourth.
* **Unit tests**: `cargo test --lib face` (the blend: weights, scales, the order, the skipped bone,
  the nlerp, the names, the blink's ease weight), `cargo test --lib audio` (the track rule against
  take07's log, a short track's fraction, the writes of a line's tracks, a speech block read) and
  `cargo test --bin bf_play faces` (the blink schedule at several frame rates and intervals, the
  controller's idle and test pose, a line's weights over its length and its end, the chatter's line
  put back a frame, a line without tracks, the stop, the expression fade).

## What the takes show for the next slice

`todo/255-facial-animation/notes.md` has each take's face log (15 Hz: the speaking byte, voice
handle, target and shown expression, the line's priority, the controller's clocks, the blink clock
and the 22 weights). The lines' (take07's dialog line, take04's chatter lines) are used above. For
the expressions' slice (#326; the fade is built, nothing sets an expression):

* **Hurt lines flick the expression to GREAT_PAIN for about 0.27 s** (take06). This is **against the
  leads** (§3.2), which read the chatter `<expression>` events' zero default duration as a no-op and
  expected no wince. Each of three hits on Tex (30 of his 115 hp at 3.0, 14.0 and 25.0 s) made him say
  a hurt line (the sounds e7c23ef7 and fdb8ec99 at priority 3, f4bfdc0f at 8), and his target
  expression (+0x58) went to GREAT_PAIN (3) for about 0.27 s (3.21 to 3.48 s), the shown one (+0x5c)
  following 0.13 s later, then both back to BASE, the speaking byte set throughout. In the frames the
  mouth opens at 3.45-3.55 s and the brows lift a little at 3.50-3.55 s: a small expression change,
  not only a mouth. (The hits were the trainer's `!hit`, which doesn't go through the chatter gate:
  only the sound events are in the chatter log, not the gate's calls.)
* **Tex's berserk face** (take04): his target and shown expression go to ANGRY (0) at 5.37 s for a
  toggle pressed at 4.0 s and stay; at the toggle off (19.0 s) the target goes to BASE at 19.44 s and
  the shown one at 20.04 s. The angry weight (15) rises linearly 0.06 (5.37 s), 0.98 (5.91 s), 1.00
  (6.11 s) and falls 0.94 (19.44 s), 0.02 (19.98 s), 0 (20.04 s): 0.5-0.6 s each way, 0.12-0.13 a 15 Hz
  step, the 2.0 a second the code gives. His berserker body fills the frame as it turns on, so the
  angry face itself isn't in the pictures.

## Not done, and what's guessed

* **The expressions** (#326): the fade (`Face::fade`, FUN_001438d0) runs each tick and feeds the
  blend's test, but nothing sets an expression yet: Tex's berserk (ANGRY on, BASE off), the
  e06 expression-trigger and the chatter lines' `<expression>` events (take06's GREAT_PAIN flicks).
* **A dialog line on its character's voice**: the game's FUN_001267b0 also sets the character's line
  priority to 10 and its speak stops the voice's chatter sound (FUN_00143570), so a chatter sound of
  a lower priority isn't started on it meanwhile (FUN_0013fab0 via FUN_00126810). The demo's dialog
  line moves its speaker's face but plays as the triggers' voice, and the chatter doesn't see it
  (chatter.md, "Not done"): a chatter sound started on the dialog's speaker during a dialog line
  without the hush flag (one in the tutorial, h_1fe7310a) would take the face from it. Also read in
  the game's code, not seen in a take and not modelled: a dialog line with the hush flag whose
  speaker's chatter line is still playing is cut by its own hush (inside its speak, FUN_00141100's
  cut-off stops that chatter line's sound event, whose stop, 0x13fb90, stops whatever the controller
  says: the dialog line just begun).
* The chatter path's first frame: the game's mouth reads its line at dt in the frame the sound
  starts, the demo's from the next frame (at 2 dt, where the game's second frame reads it). One frame
  at the start of each chatter line, which shows on a line that starts mid-shape (h_05410be5: `size`
  0.77 at its start).
* The transition's blend of the lids (the channel's own, its vtable +0x24) isn't read: a normalised
  lerp is used (inferred).
* The blink's cycle, its phase by the character's first tick, and Hawk's missing blink are measured
  now (above). Still inferred: the held last key after a blink (nothing resets the nodes in the code
  that was read; no take could read it). Brutus's lids are unread (take02).
* The first blink after a spawn was not caught on screen (no take logged one): the staggered phases
  are what show it.
* A knocked-down but living character's face isn't ticked, and during the get-up the step runs twice a
  frame (a gap in the knock-down's code, `update_player`), so its blink clock runs fast for a second.
* A face bone missing from the model would stop the game's writes altogether (FUN_00143820's count
  test); the demo leaves that bone out. None of the data's models misses one.
