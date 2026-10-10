# Squad: movement, AI, hits and deaths

Squad movement and deaths follow the game's own data and the captures:

* Wingman follow (the squad AI's follow personality, decoded from common/ai/personality.xml
  with the event / goal enums from default.xbe): squadmates keep loose places 6-11 m *ahead*
  of the leader (measured from the captures), set off when 3 m from their place and settle
  within 1 m, dash when far behind (or past the data's squad-leash-dist + 10 m), and face the
  leader's heading. Standing a while, Flint, Hawk and Tex kneel (stand2crouch -> rp_crouch_idle,
  crouch2stand to move on); Brutus stays on his feet. They step out of the lane you aim or fire
  down (EVT_SHOT_BLOCKED_BY_FRIEND), sidestep now and then when you shoot them
  (EVT_DAMAGED_BY_PC -> GOAL_DODGE) and dive away from a grenade that lands near them
  (EVT_GRENADE_NEAR -> GOAL_DIVE, the motion scripts' dive clip).
* **Standing on the floor.** The floor is 1.05 m (`GROUND`) under each character's pelvis, but
  the four aren't the same height. So each is raised or lowered by how far the lowest
  skinned vertex of their idle pose is from it (`sole_lift`, up to 0.3 m either way):
  * Tex: raised 0.17 m (his soles went through the floor);
  * Brutus: lowered 0.07 m (floated);
  * Flint: about right (0.01 m);
  * Hawk: lowered 0.15 m (floated).

  That's for standing. A kneel's lowest point (the knee) isn't the standing soles', so each
  crouch clip has its own lift (`clip_lift`): `stand2crouch` and `crouch2stand` through their
  length (16 samples), and `rp_crouch_idle` and the crouch walks as the most over their cycle.
  The model is lifted by the weighted mix of what's playing, so a kneel or a crouch walk
  stands on the floor too. With the standing lift alone, Hawk and Flint knelt in the air. This
  covers the squad's kneels as well.
* **Scopes.** With a weapon that zooms, right mouse toggles its scope (click in, click out;
  other weapons still aim while it's held). Flint steps in twice: in, closer (the weapon's zoom
  doubled: the L-Shot-50's 5x, then 10x), out. The doubling is a choice; the data has one zoom
  per weapon. Switching weapons or reloading drops out of the scope. `BF_TEST_SCOPE=<step>`
  holds a step. A scoped shot leaves from the eye along the crosshair's ray (from the
  muzzle, a little lower, it would land low through the zoom). In the scope the camera
  moves to the character's eyes: the characters' `offset-snipe` camera offset is 0.001 0 0,
  against 3.2–3.4 m behind for walking, running and ready. The field of view narrows by the
  weapon's zoom (`h_efce7f77` on its definition: MK-ASLT 2, Kman Auto 2, Psionic Blast 2,
  Foley 356 Tact 3, L-Shot-50 5, L-Shot-75 10; 0 on the rest), the model is hidden, and mouse
  look slows by the same factor. Shots still go through the crosshair. While scoped, the
  character's own breathing loops: each character's `snipe-sound` (`h_12501487`, beside its
  radar-range), a 2D sound of the common bank (Brutus `h_01a51dd2` 2.9 s, Flint `h_15545426`
  0.7 s, Hawk `h_1ca2a830` 1.7 s, Tex `h_0ce9616b` 1.8 s), with 0.6 s between breaths.
  `BF_TEST_AIM=1` holds aim.

  Two kinds of scope view, as in a capture:

  | Who | View |
  |---|---|
  | Flint (the sniper) | Fills the screen, with a cyan-teal glow at the edges. Her aim is dead steady (she's a synthetic). The game smears those edges with an effect, so the glow here is an approximation. |
  | The others | The view sits inside the game's scope frame `h_1807ec04`: a 256 × 128 quarter (rounded corner, tick marks) mirrored four ways. It's sized so the rim spans 77% of the width, as in the capture. The texture's grey outside is drawn darkened. |

  The others' aim wanders while scoped: a slow drift of up to about 0.7° that moves the view
  and the shots together. In the scope everyone can still move, but only at a walk. The
  crosshair moves to the middle of the screen and grows from 40 to 72 units. The eye stays
  0.25 m off whatever is in front (by the collision), so a close wall can't be looked through.

  Going in and out plays the character's two scope sounds. They're stored beside its
  snipe-sound: `h_00dd1fe0` and `h_02c94d34`, about 0.3 s each, taken as in / out. Flint has
  her own pair (`h_1fd08ea5`, `h_03e0936d`); the others share `h_135ec2ce`, `h_f9ffc972`.
* **Squad hand-over.** It plays one sound as the camera cuts: `h_07949f0d` (common bank,
  0.18 s), found in a capture at all five hand-overs. It replaced the weapon-switch pair used
  before. The name is drawn 1.6x the capture's size, pops in from nearly double that, and at
  the cut swells to 6x while it fades over 0.4 s; the game's swell is about 3x, exaggerated
  here as asked. `BF_TEST_SELECT=<character>[,<s>]` picks that member at 1 s (or s; control passes 0.6 s later), also together with `BF_TEST_GOTO`.
* **The squad's AI table.** Squadmates follow you as wingmen. Their personality
  (`common/ai/personality.xmb`; each character's own `personality-wingman` is empty and
  inherits `h_ea20a17d`) maps events to goals, with priorities and weights. Its events and goals
  are numbers. The names come from default.xbe's name tables: events at 0x3bdc14 (`EVT_DEAD` =
  10, … 96), goals at 0x3bce78 (`GOAL_DODGE` = 59, … 134). Without enemies, these entries
  apply:

  | Event | Goals (weight) | Here |
  |---|---|---|
  | `EVT_GRENADE_NEAR` (pri 28) | `GOAL_DIVE` | dive away from a grenade that lands near |
  | `EVT_SHOT_AT` (20.15) | `GOAL_DODGE` 1, `GOAL_NOOP` 7 | a shot of yours passing within 1.5 m (the distance is a guess): sidestep one time in eight |
  | `EVT_DAMAGED_BY_PC` (15) | `GOAL_DODGE` 1, `GOAL_NOOP` 2 | shot by you: sidestep one time in three |
  | `EVT_BASE` (1) | `GOAL_WINGMAN_FOLLOW` | the formation |
  | `EVT_IDLE_VARY_DATA` | `IDLE_VARY_DATA_PAUSE_CROUCH` 15, `…_PAUSE_CAUTIOUS` 12 (equal weights) | holding position: the first pause kneels (as in the captures), then kneeling and cautious pauses (standing, weapon ready, looking left and right) are drawn, lasting 15 and 12 s |

  `BF_AI_LOG=1` prints their idle choices and dodges.
* Hits come from the game's effect library (common/effects-common.ale, read with
  `../ale_tool.py`) and the flesh world material (21, WMAT_FLESH_HUMAN). Every hit plays its
  debris bundle: blood_puff (smokecard.tga mist), bloodsplat_s (diffuse_big.tga droplets in a
  jet, under a weak gravity field) and giblet (a blood.tga spatter). Each ammo type adds its own
  hit effect and sound: ballistic sparks (Flint, Hawk), the cutter's long white streaks with a
  pale blue flash light (Brutus) and the laser's orange embers with an orange flash (Tex). The
  emitter / appearance numbers in `play_fx.rs` are those nodes' parameters, read as Freelancer's
  ALE ones (Brute Force hashes their names). Each hit also leaves the character's hit decal on
  the ground past them (h_14410af1: the game's white splat textures tinted 128,34,34,220; about
  1 m across, as in the captures) for 40 s, fading over the last 20. Flint bleeds grey: she's
  taken to be synthetic flesh (world material 24, WMAT_FLESH_SYNTHETIC: an inference from her
  grey decals, the data doesn't name a character's material), whose debris bundle h_1f7ca034
  is blood_puff_b, bloodsplat_b and giblet_b - the same nodes in grey and blue-grey, slower
  and smaller - and whose hit sounds for the heavier ammo are metallic. Her hit decal
  (h_02d017b7) is tinted 50,50,50. Verified with `BF_CHARACTER=1 BF_TEST_SUICIDE=1 -- --test`
  captures against Brutus (`BF_CHARACTER=0`): grey mist and splats for her, red for him. A
  hard hit (15+ damage: L-Shot, Bower, a close grenade) can knock a character down: the body
  goes limp (ragdoll, pushed harder at the top so it topples) for 0.7 s, thuds on the ground
  (the surface's landing sound) and gets back up over 0.45 s where it lies - the game uses
  Ipion physics for this (its constraint messages are in default.xbe, with MT_KNOCKDOWN).
* Death: the death cry (chatter e1213623 - the captures' deaths match its lines), a thud, a
  limp body that stays on a blood pool (the character's other decal, h_16d327fd), blood
  splashed round where they fall (the hit's blood effects sprayed up, and five of their hit
  splats on the ground within 1.4 m: the demo's choice, the game's own death blood isn't
  known; Flint's decals are dark grey, 50,50,50, in the data: she's a synthetic), their memory
  chip beside where they fell (below), and - half the time, as the data says - a surviving
  squadmate's answer from the dead member's response tag ("Brutus is down!", "They've taken out
  Hawk."). On the radar the portrait becomes the member's own skull (the characters'
  h_00c51907 icon) and the member's tab and health bar go.
* **Memory chip** (`play_dna.rs`, #43; what the demo called "the DNA"). Each squad
  character-object's `<h_0f77963d inventory-drop>` is item h_f50f94f9 (function-type 18,
  h_0a811e94 2000, message h_1dd3a49d -> h_091ce603 "Memory Chip Recovered!", pickup-sound
  h_ee2c16da). Its mesh h_10960346 is a 0.375 m cube, UVs 0..1 on each face, in the self-lit
  shader h_f539fe8c: the circuit-trace texture h_1e02a5ef (two thirds alpha 0) times its
  h_e01baa40 colour 0.36 1 0.67 at alpha 0.7, blended SRCALPHA / INVSRCALPHA without z-writes
  and unculled (the shader's render-state setup, FUN_0008fd20; h_0f5ae13f 1 would be additive,
  the chip has 2), so the far faces show through the near ones. Its scroll (FUN_0008fca0)
  only runs for h_08c2d2ee = 0.12 s, so the texture sits at a fixed offset (0.446, 0.102). It
  appears 0.2 s after the death, 0.9 m from the body, its middle 0.55 m over the ground (the
  demo's choice, kept from the old effect; the game's isn't found), and doesn't spin or bob (as
  in the footage). The player walking into it (within 1 m across, 1.5 m up or down) takes it:
  it goes, h_ee2c16da plays and the pickup lines show "Memory Chip Recovered!" (no count). The
  footage's "+ 2000" by the radar is the score, which the demo doesn't keep; nor does it
  reclone the squadmate. The green
  sprites and blue light the demo drew before were effect h_ee11d51f (powerup_pill +
  light_powerup_pill), which belongs to the DNA canister pickup (mesh h_e3e4caad, "Alien
  Technology Acquired!"), not to a death. (`BF_PICKUP_LOG=1` prints the chip's material, where
  each is dropped and when it's taken.)
  * Its pixel program (pixel shader 0, defined at 0x3ddf80 in default.xbe, set from
    FUN_0008fd20 through FUN_000a4d00) is one combiner stage, texture x constant c0 for colour and
    alpha alike, c0 being the h_e01baa40 colour with `alpha` as its fourth component
    (FUN_0008fa50 maps the names to +0x70 / +0x7c; FUN_000a4d80 packs them), then a final
    combiner that fogs the colour. No vertex colour, lighting or second texture. So it's drawn
    as Bevy's alpha blend of that, fogged, with a box-filtered mip chain made for its texture:
    the file's own five levels (64 down to 4) are box averages too (mean alpha 39, 49, 52, 43,
    40 of 255), but the format reader decodes the top level only. Without mips the one-texel
    traces break up a few metres off.
  * Not used: `time-scale` (h_01590d7a, 0.4) is mapped to +0x58, but none of the shader's own
    functions read it (and it couldn't move where the scroll stops). The wrapper's h_e59d69a0
    (60) goes to render state +0x294 of the state cache (FUN_0009e960, flagging the material
    when it isn't 255); which state that is isn't established. An alpha-test reference is a
    guess, and as one it can't be cutting at 60/255, since the footage's distant chip is filled.
  * **Not matched: the footage's fill.** Far off (Friendly Fire 2, 33.8 s) the chip's inside is
    green +58 over the background, red and blue up too; the demo's is green +30 to +48 at 6-15 m
    on sdm_e34, red and blue a little down. Up close in front of a lit wall (DNA + Weapon
    Pickups, 12.4-12.6 s) the footage's is whitish cyan, red 165 over 60, with a soft halo.
    The pixel program can't raise red above the background (texture red 45 x 0.36) and the
    texture's mips don't fill it, so that light comes from something outside the chip's
    material: not found. In the DNA footage the Light grenades' beams are close by, and at
    9.4 s a soldier walking through the chip is lit the same whitish cyan. A guess, not drawn.
  * Where it lies: at the moment of the take the footage's chip shows above Hawk's head and the
    demo's at Tex's hip. That may be the slope, not the height; not fitted.
  * It sorts 1000 m nearer among see-through things than it is: the terrain's
    blended texture layers are drawn in the same pass, sorted by their chunk's middle, and
    painted over it on sdm_e34 (it all but vanished: a 7/255 difference). Known limit: it now
    sorts after every other blended thing (gas clouds, blood mist, liquids, see-through
    materials), so it shows crisp over a cloud between it and the camera. (The same value is
    also the pipeline's depth bias, about 1e-4 of the depth: it doesn't draw through walls.)
  * `BF_TEST_CHIP=<x>,<z>[,<s>]` drops a chip at (x, z) at that time (default 0.5 s), on the
    floor below 2 m over the player's middle, without a death.
  * Verified: a squadmate killed on the test map (`cargo run --bin bf_play -- --test` with
    `BF_TEST_GOTO=0,0,0,0 BF_TEST_KILL=0`: Brutus's chip 0.9 m beside him, a see-through
    circuit-trace cube, its far faces through the near ones); on sdm_e34 standing
    (`BF_TEST_GOTO=-44.4,15.5,-44.4,15.5 BF_TEST_CHIP=-43.2,13.0,0.5`, with `BF_VIEW_YAW=0` and
    `0.6`: one face square-on, then two faces and an edge, as the footage's camera pass at
    2.5-6.5 s) and running through one (`BF_TEST_GOTO=-44.4,15.5,-44.4,35
    BF_TEST_CHIP=-43.8,22,0.3`: it goes, h_ee2c16da plays, "Memory Chip Recovered!" shows).
* **Death camera** (`play_deathcam.rs`, from the reference recording `Friendly Fire 2 + Death
  Cam.mp4`, 28.3-32.4 s). On the frame the character you control dies, the view cuts (no
  blend) to a camera on the body and the reticle goes. The camera uses the character type's
  `offset-dead`: camera block `h_0d41e5f1`, element f13fb0c4, `3.5 5 0` for the squad. The
  objecttypes parser `FUN_0018fa80` stores the seven offsets at +0x1b0 in the character type,
  and all seven are now read into `Game::character_camera`. The camera sits 3.5 m above the
  ground under the pelvis and 5 m back. It aims 11 degrees above and 6 degrees left of the
  pelvis, so the body sits low and right of centre, at about (380, 350) of 640 x 480 as in
  the recording (both angles measured). It circles the body counter-clockwise seen from above,
  travelling to its own right, at a steady 60 degrees per second. It starts from the follow
  camera's heading. The rate is an estimate: 55-66 degrees per second measured through the
  recording's gas smoke.

  Walls: the game's own camera collision here isn't recorded, so this is the demo's choice.
  The camera sits on the line of sight from the body (0.3 m above the pelvis) to where
  `offset-dead` puts it, and keeps 0.3 m in front of whatever the level has on that line, so
  rock never hides the body. It eases in ahead of a wall, looking 20 degrees along the orbit.
  It never comes closer than 2.5 m: where a wall leaves less room, that part of the circle is
  skipped with a cut ahead to the next heading with room.

  On the death frame a squad switch picked just before is dropped, so the death camera always
  runs its full 4 s. A scope ends at once (no zoom easing out, the body not hidden).

  At 3.5 s the next living member's name appears over it, at its final size from its first
  frame, as in the recording (f1910 and f1911 are the same). A manual switch still pops it in.
  At exactly 4.0 s the squad switch cuts to that member's follow camera, and the name swells
  and fades across the cut. Keys 1-4 do nothing while you are dead. With nobody left to take
  over (the whole squad down (#41), or deathmatch) the follow camera stays on the body as
  before.

  The hand-over, for a death or a manual switch, cuts to the camera straight behind the new
  character's facing, the whole body in view, as the recording's cut to Hawk does. If a wall
  is there, it swings in 10 degree steps to the nearest heading either side with room;
  before, the follow camera's clamp could wedge it inside the body. The new character's aim,
  fire, walking and squad-AI idling (the leader's heading it turned to, kneeling) are dropped,
  so the cut is to the plain follow camera with them standing still. Standing a kneeling
  member up is the demo's guess.

  After the cut the follow camera is unchanged: it stays on the heading the cut chose until
  you turn it. If you turn it toward a close wall, its clamp still pulls it in, as close as
  0.2 m. That behaviour is left for #89.

  Not done: the HUD's red tint and fade, the red score popup, steering with the right stick.
  `BF_DEATHCAM_LOG=1` prints the camera every frame (heading, how much was skipped at walls,
  distance and room). `BF_TEST_DIE=<s>` kills the player once per level.

  Verified at 15 fps on sdm_e34 (`BF_MAP=sdm_e34 BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=<n>`, plus):
  - `BF_TEST_GOTO=15.3,37,15.0,29 BF_TEST_DIE=1`:
    - frame 15 has the reticle on;
    - frame 16: Tex dies, the view cuts and the reticle goes;
    - frames 16-75: 60 frames turning 4 degrees each;
    - frame 69: "BRUTUS" (t = 3.53 s);
    - frame 76: the cut to Brutus, from behind and whole. After that the test hook's autopilot
      turns the camera toward its goal.
  - `BF_TEST_GOTO=16,37,16,37 BF_TEST_DIE=0.5`, against the cliff:
    - the body stays in view at 5.5-5.9 m;
    - at t = 3.6 s (frame 62) the orbit skips 130 degrees of cliff and lands 2.8 m out;
    - frame 68: the cut to Brutus from behind.
  - `BF_TEST_DIE=1` from the map's start: the cut at frame 76 is to Brutus's back, and he
    stays facing away.
  - `BF_CHARACTER=1 BF_TEST_SCOPE=1 BF_TEST_DIE=2`: Flint scoped through frame 29; frame 30
    cuts to the unzoomed death camera with the body in view.
  - `BF_TEST_SELECT=0 BF_TEST_DIE=1.2`: the manual label shows at frames 16-17, Tex dies at
    18, and the label goes. The death camera runs its 60 frames. "BRUTUS" is the same size at
    frames 71, 72 and 73, and the cut is at frame 78.
  - `BF_TEST_SELECT=0`: the manual switch still pops in (frame 16), and frame 25 cuts to
    Brutus from behind.
  - `BF_TEST_GOTO=16.5,37,16.5,37 BF_TEST_DIE=0.7`, a cliff behind Brutus at the hand-over:
    - frame 71: the cut, from behind with his whole body in view;
    - from frame 72 the hook's autopilot (a `BF_TEST_GOTO` whose goal is its start) sets the
      camera heading to 0 every frame, which points it at the cliff, 1.8 m away;
    - the follow camera's clamp then pulls it in close to him (frames 73-89). That comes from
      the hook and the clamp, not the hand-over.
  - With `BF_DEATHMATCH=1` there is no death camera.
* `BF_COMBAT_LOG=1` prints hits, knockdowns and dives.
