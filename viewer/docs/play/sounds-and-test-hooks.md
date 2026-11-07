*Part of [play/](README.md), the former page `play.md`.*

Sounds (`src/bf/audio.rs`) come straight from the game's banks: `sounds-<level>.xmb` maps sound
ids to Xbox ADPCM data in `sounds-<level>.mem`, decoded to 22 kHz WAV in memory. Each character's
archetype names its jump grunt and footstep type (Tex 1, Flint 2, Brutus 3, Hawk 4), and
`world-materials.xmb` gives every surface four footstep variants per type plus landing and slide
sounds. Footsteps fire when a toe comes down into the lowest quarter of its swing in the clip
being played, each one FUN_00119020's request (**Footsteps (#324)**, below: the surface's pick, a
volume by the gait, 2D for the controlled character and at the foot for the others, a random
pitch); in a liquid the step is the liquid's wading step or nothing, by the depth
(FUN_00119020's liquid branch, #5: levels.md, "What a liquid plays"). The footsteps' and the
liquids' sounds are requests with play parameters of their own (`WorldSound`: a position heard
by the library's distances from the listener and dropped beyond its max distance, a volume, a
pitch in octaves played as the speed); `BF_SOUND_LOG` prints each with its kind, volume, a
footstep's gait factor, pitch and distance. Surface sounds live in level archives: `BF_LEVEL=<name>` (default `e01`) picks the
level; `mp1`'s bank is merged in too, since a mission only carries its own squad's footsteps. They
load after the played map's own data (`load_map`, `load_default_level`) and only fill in what it
lacks: where two levels name something alike the map's is the one found (#338, levels.md "Which
level's data").
On a level, its sound-triggers play their own voices (#223: voice lines, door and gate sounds,
ambience, the music cues; 3D ones by DirectSound's distance law from the camera): see
[triggers.md](../triggers/sound-and-dialog-triggers.md), "Sound- and dialog-triggers", and their `BF_SOUND_LOG` lines
("trigger voice ...").

Test hooks: `BF_AUTOPILOT=1` plays a scripted run (idle, run, turn, sprint, aimed walk,
backpedal, dodge, standing jump, running jump, aimed fire, weapon switch, firing on the run); `BF_TEST_EXPLOSION=1` sets off the selected grenade type 6 m ahead every 1.5 s; with `BF_TEST_GOTO`, `BF_TEST_THROW=<s>[,<hold s>,...]` holds the grenade button (G) at those times (see **Grenades** for the rest of its hooks); `BF_ANIM_PROBE=<script>,...` prints each character's clips by script name (duration, root motion over a cycle, the clip's own speed h_f9628571, events); `BF_DUMP_TEXTURE=<file>:<hex id>` writes a decoded texture (raw RGBA after a u32 width and height). `BF_AUTOPILOT_SPIN=<rad/s>` makes it look down and turn the camera during the standing fire; with `BF_CAPTURE=<dir>` it saves every frame at a
fixed 15 fps (`BF_CAPTURE_EVERY=<n>`: only every n-th, for a long run at that step whose logs
are what's wanted, #294). `BF_CAPTURE_EARLY=1` captures from the map's first frame instead of after the warm-up and the wait for the character and the LOADING cover (#311, [menu.md](../menu.md) "The level's start behind LOADING"): a capture otherwise starts once the cover is down, and so at the start of the level's fade from black (#266, [mission.md](../mission.md) "A level's start": its first second is the ramp; `BF_NO_FADE=1` leaves the fade out, and since #390 a script's 120 fades too: [cutscenes.md](../cutscenes.md) "The screen fade"). `BF_MUTE=1` silences audio, `BF_SOUND_LOG=1` prints every sound event with its time
and action, `BF_DUMP_SOUND_IDS=<dir>:<id,id..|all>` decodes sound ids to WAV, `BF_DUMP_SOUNDS=<dir>` writes the character's jump and surface sounds as WAV files
and exits. `BF_DUMP_WEAPONS=1` lists every character's weapons as loaded (definition, model parts,
hardpoints, muzzles, fire sounds), then the map's definitions with more than one muzzle, and exits; `BF_DUMP_TYPES=1` (or `all`: every archive) lists the
character types, their motion sets' clips and the placements, and exits
([character-types.md](../character-types.md)); `BF_SHOT_LOG=1` prints each shot's barrel and shot
directions; `BF_FEET_LOG=1` prints, every frame, the controlled character's soles (the posed
model's lowest skinned vertex) and its two foot bones over its ground, whether it moves, and the
overlays playing with their times (a pickup, a use, a reload, a throw: #293's check of the
pickup clips' feet). Camera hooks ([camera.md](../camera.md)): `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` set the
start (with `BF_TEST_GOTO`, `BF_CAMERA_PITCH` holds the pitch throughout, e.g. to fire down at
something; held to +-60 degrees), `BF_CAMERA_DISTANCE` sets every character's boom,
`BF_CAMERA_AIM_AT=x,y,z` turns the aim so the crosshair's ray passes through a point (the recipes
here from before #126 aimed with `BF_CAMERA_PITCH` on the old camera: the same pitch now puts the
crosshair elsewhere), `BF_CAMERA_LOG=1` prints the camera, `BF_VIEW_YAW` turns only the rendered
view about the pivot (aim unchanged), `BF_START_WEAPON=1` starts with
the second weapon. Captures wait 60 frames for shaders to compile before the clock starts.
