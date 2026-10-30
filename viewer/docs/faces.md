# Faces (`bf/face.rs`, `src/bin/play_faces.rs`, #255)

How a character's face is drawn in play: the game's face controller, not a clip. This is the
first slice of #255: **the face table, faces at rest and the blink**. The lip-sync (a speaking
character's mouth) and the expressions come in their own slices; the controller already has
their hook, the 22 viseme weights (below). Findings behind it: the code map's "Faces" section
(`code-map.md`); the analyst's leads are `todo/255-facial-animation/leads.md`.

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
the clip is `Sc_` + the name; `clip_name` gives a clip's name for the viewer), and the head bone's
hash and the height of a face over it for the close-up cameras.

## The controller in the demo (`play_faces.rs`)

`Player::facial` (`faces::Face`) is the controller: **`weights`** (22 floats, the hook the next
slices write: the lip-sync into 0-14 as the sound's tracks run, the expression fade into 15-21),
the face bones' output pose, and the blink's clock. It is made as the character is
(`spawn_unit`, `faces::spawned`) and ticked by `step_player` (`faces::step`) after everything that
poses the body: the clip layers, the weapon switch, the reload, use, throw and skill overlays and the
aim's turn. No body pose carries a face clip any more (`Character::pose` is called with no face:
the game's body clips don't move these bones), so the face and the body never fight: the
controller's pose is written over the bones last.

* **Tick** (`Face::tick`): no time, nothing. While a weight is up the blend runs (FUN_001439b0); with
  every weight 0 nothing is written and the face keeps its last pose, as the game's idle controller
  does (the lip-sync slice swaps "a weight is up" for the game's test, a line speaking or an
  expression that isn't BASE, which also writes the base through a line's silences).
* **At rest** the face is `Sc_base` on the face bones from the first frame.
* **Alive only**: `step_player` runs for a living character, as the game ticks the controller from
  the character's update (0x11dc7b: bit 0 of char+0x70 clear). A dead character's ragdoll starts
  from the pose it last stood in (`last_world`), so its face stays as it died; a knocked-down one
  that is still alive isn't ticked in the demo (the game's would blink on, a gap).
* **Cutscenes**: a held character is stepped like any other, so Tex in the second patrol's close
  shot blinks on its schedule.

### The blink

FUN_00143d10: a clock (+0x4c) of the time since the last blink, **starting at 10 s** (FUN_00143380),
counts up each tick; once it passes the clip's length (0.333 s) and **the type's interval** (h_09109ae2:
4 s for the squad), `Sc_blink` plays once and the clock is zeroed. So the first blink is at the
character's first tick, then one every interval plus up to a frame: at 15 fps (BF_CAPTURE) 4.07 s,
in real time 4.0 s. Between the clip's length and the interval the game tests a random number against
the chance (0.05), but its number runs 0 to 131072 (never scaled to 0-1, then divided by dt x 30),
so the test passes about once in 2.6 million frames: the demo doesn't have it, and the blink is a
regular cycle (the random part is dead, by the code; footage B1 of the shot list should confirm the
cycle).

The clip plays on its own bones: the lids, which aren't in `Sc_base`'s list, so the controller doesn't
own them. After the 0.333 s the nodes keep the clip's last key (a held pose, what the game's nodes
do when nothing writes them again; inferred). Before the first blink the lids are the model's own
(the bind pose) and the first tick ends that.

**Hawk doesn't blink.** The face controller flags every face bone's node, the binder skips flagged
nodes, and Hawk's `Sc_base` lists her four lid bones (the 22 bones of her list: four more than Flint's
and Tex's). So by the code `Sc_blink` finds no node to bind on her: it plays, and nothing moves. (Her
`Sc_base` holds her lids' rest values, equal to the blink's first key, so they sit right.) The demo
leaves those targets out and so shows none; the code says so, no footage has been looked at for it.
The shot list's B1 should include her.

How far the lids' rest keys lie from the bind pose: none for Flint and Tex (0.0 degrees), 20 and
7.5 degrees for Brutus (his eyes narrow when the first blink ends).

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
  the chance, the stretches) and each blink as it starts (`t=4.07 tex: blink 2`); with `BF_TEST_FACE`
  each weight set.
* `BF_FACE_CHECK=1`: for each face made, how closely its table reproduces the clips it came from: the
  base and the 22 poses (each viseme at a weight of 1 over its scale) against `Character::pose` with
  the clip laid over the same body pose (the viewer's way), on the bones the clip names: the largest
  turn and move, and the lids' first and last key against the bind pose.
* `BF_TEST_FACE=<s>,<who>,<viseme>,<weight>[;...]`: at `s` seconds (the controlled character's clock) set a
  viseme's weight on `who` (`me` the controlled one, a squad name or 0-3, `all`). It stays until
  another entry changes it. `--test BF_CHARACTER=3 BF_TEST_FACE="1.0,me,roar,1;1.4,me,roar,0;1.4,me,angry,1"`.
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
* **A cutscene** (`BF_MAP=tutorial BF_TEST_GOTO=34.5,75,34.5,67.5`, `tutorial_show_patrol_2`'s close
  shot of the held Tex): the face intact, the blink at 4.13 s on the camera 5 m away.
* **A death** (`BF_TEST_DIE=1.0`, from above): the face as it died, no stretched geometry, the head
  thrown back.
* **Unit tests**: `cargo test --lib face` (the blend: weights, scales, the order, the skipped bone,
  the nlerp, the names) and `cargo test --bin bf_play faces` (the blink schedule at several frame
  rates and intervals, the controller's idle).

## Not done, and what's guessed

* **The lip-sync and the expressions**: the library's viseme tracks (`bf/audio.rs`), the speak and
  stop calls on every speech path, the expression fade and Tex's berserk face are the next slices.
  `Face::weights` is where they write.
* The blink's cycle and Hawk's missing blink are the code's reading, not footage's. The held last key
  after a blink is inferred (nothing resets the nodes in the code that was read). The play request's
  fields (a 0.1 beside the rate of 1.0 that FUN_00143d10 sets) don't blend the clip in as far as the
  code that binds it shows.
* A knocked-down but living character's face isn't ticked, and during the get-up the step runs twice a
  frame (a gap in the knock-down's code, `update_player`), so its blink clock runs fast for a second.
* A face bone missing from the model would stop the game's writes altogether (FUN_00143820's count
  test); the demo leaves that bone out. None of the data's models misses one.
