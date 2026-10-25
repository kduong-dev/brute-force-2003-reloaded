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
* **The test map's squad** (#302, play_testsquad.rs, [testtools.md](testtools.md) "The squad"):
  it starts with the player's character alone, and its squad menu (T) puts any character type in
  any of the four slots. The slot (`Player::character`) is the place on the HUD, the key and the
  identity for the hand-over; what this page reads by a character's name (chatter, skills, the
  camera, Brutus not kneeling) goes by its type (`Player::data_slot`), the same in the main game.
* **The tutorial's Confed marines** (#227, [brain.md](brain.md)): the V-squad's two follow Tex
  with this follow, from the formation places after the squad's own, see by their sensor (80 m,
  90 degrees either way) and fire with the AI's aim (accuracy 0.6). In take11 they trailed Tex
  and stopped 9-10 m behind him; the game's wingman follow is a state machine of its own
  (FUN_00020b10) the demo doesn't read. Switching to them (the d-pad) isn't done.
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
* **Scopes.** With a weapon that zooms, right mouse is the scope button (in, out; other weapons
  still aim while it's held). How many levels is the holder's skill's: one for Tex, Hawk and
  Brutus, two for Flint, who steps in, closer, out, with any gun that zooms; her second level is
  z + max(10, z + 5) (the L-Shot-75 10 then 25, the L-Shot-50 5 then 15), the code's rule, which
  the takes confirm (play.md, "The scope"). The zoom ramps linearly, 0.15 s a level, and the
  button waits out a short lockout after going in or out. Switching weapons or reloading drops
  out of the scope, and Tex's berserk keeps him out of it ([skills.md](skills.md)).
  `BF_TEST_SCOPE=<level>[,<from s>]` holds a level (from that time on), `BF_TEST_SCOPE_PRESS`
  presses the button (play.md). A scoped shot leaves from the eye along the crosshair's ray (from the
  muzzle, a little lower, it would land low through the zoom). In the scope the camera
  moves in by the scope's progress to 0.5 m behind a point 0.5 m above the character's root
  node, every character's (the follow camera's own scope rig, 0x128280; the types'
  `offset-snipe`, 0.001 0 0, isn't read: [camera.md](camera.md), "The scope"), and the aim
  turns so the crosshair stays on its point as the view moves. The field of view narrows by the
  level's zoom (`h_efce7f77` on the weapon's definition: MK-ASLT 2, Kman Auto 2, Psionic Blast 2,
  Foley 356 Tact 3, L-Shot-50 5, L-Shot-75 10; 0 on the rest), the model is hidden, and mouse
  look slows by the same factor. Shots still go through the crosshair. While scoped, the
  character's own breathing loops: each character's `snipe-sound` (`h_12501487`, beside its
  radar-range), a 2D sound of the common bank (Brutus `h_01a51dd2` 2.9 s, Flint `h_15545426`
  0.7 s, Hawk `h_1ca2a830` 1.7 s, Tex `h_0ce9616b` 1.8 s), with 0.6 s between breaths.
  `BF_TEST_AIM=1` holds aim.

  Two kinds of scope view (#283, `src/bin/play_scope_view.rs`). Which one is the character's
  skill's (its +0xa8): with bit 0 set (Flint's, 0x2d5e40) her overlay, with it clear the frame
  (everyone else's, Tex's 8 too: nothing reads bit 3), so it follows her skill whatever gun she
  scopes with (#62). Both are HUD widgets the crosshair widget's update makes in code (0xc6030;
  they aren't in the HUD's layout data), under the rest of the HUD:

  | Who | View |
  |---|---|
  | Flint (the sniper) | Fills the screen, with her overlay (FUN_000b3520) added over it: a cyan-teal smear round the picture's corners and along its edges (`h_f5b6a9f8`) and an ellipse of teal and yellow-green dots near the edges (`h_f0d8a0a7`). It comes in over 0.17 s from the press, shrinking from 3x its size to its scoped 2x as it brightens, the same at both levels, and goes the same way out. Her aim is dead steady (she's a synthetic). |
  | The others | The view sits inside the game's scope frame `h_1807ec04`: a 256 × 128 quarter (rounded corner, tick marks) drawn 1:1 from the middle and mirrored four ways, the opening x 164-476, y 127-353 of the 640 × 480 screen. Outside the rim the picture shows at 40% of its brightness, on the rim's line at 20%, the tick marks at 76%. The frame is whole at the press; leaving the scope it grows 3x about the middle in 0.33 s, fading over the last half. |

  How they're drawn (the leads, `todo/283-scope-hud/leads.md` §2-3; every function in
  [code-map.md](code-map.md), "HUD"):
  * **The frame** (update FUN_000b4070, draw 0xb4240): scoped (char+0x7a8) with bit 0 clear, its
    scale is 1 at once; else it eases toward 3 at 6 a second, its alpha 1 - clamp(scale - 2, 0, 1).
    Four quads from the middle, the texture's first column and last row there, texture stage 0
    set to CLAMP (FUN_0006a610 on its ADDRESSU / ADDRESSV), so the texture's outer grey
    (153) carries on to the screen's edges. The sprite draw (0xba950) blends it in its mode 5,
    ZERO / ONE_MINUS_SRC_COLOR: the picture times 1 - the texture x the alpha.
  * **Flint's overlay** (update 0xb3700, draw FUN_000b3970): scoped, its scale eases toward 2,
    else 3, at the frame's rate and with its fade. Its target in the scope is (1 - the reticle
    controller's +0x74) + 1, and the controller clears +0x74 every frame (its +0x58,
    FUN_000c52f0), so 2. Each texture in four mirrored quads of 160 × 120 units x its own scale
    (floored) x the widget's, the texture's first column and last row at the picture's corners:
    the ring at a scale from 1.5 to 1.2, then the smear at 1.1 to 1.0, by how deep the scope's
    focus lies between the follow camera's near plane (0.1 m) and its far plane (the
    `clip-plane-far` of the follow camera archetype the level file names, h_e8eb1b7b:
    `Level::follow_camera`, `Game::follow_cameras`; 250 m on 35 levels, e34's and e40's among
    them, 100 on 12, up to 800 on m14_c and m14_d). The focus is the centre ray's point's depth
    (`Reticle::depth`), taken at the scope's first frame and followed at 1000 m a second
    (FUN_000702a0's cam+0x1f8): 11.3 m at e34's range spot, ring 1.49. Mode 2 adds them, ONE /
    ONE, their texels times the vertex colours, grey 30 and 50, times the alpha. Not drawn in a
    vision mode (the camera's +0xd8: Brutus's Vengar view).
  * The console blends in the frame buffer's values (sRGB); Bevy's UI blends in linear light,
    where the added colour comes out much fainter over a lit scene. So both are one pass over
    the main camera's picture (`src/scope_view.wgsl`), after the Vengar view's world pass and
    before the HUD, in those values. It goes with the HUD: not under `BF_NO_HUD`, nor while the
    HUD is hidden (a cutscene's letterbox, the map editor).
  * In a window wider than the game's 4:3 the sides go on as CLAMP addressing would carry them:
    the frame's grey to the window's edge, the overlay's outer texels past its quads (the demo's
    choice; the game's screen ends at 4:3).
  * Before #283 the frame's quarter was drawn 397 × 269 (55% too big) as a dark tint at the
    texture's alpha, blended in linear light (lighter than the game's multiply), and Flint had a
    cyan rim of the demo's own over the whole window.

  `BF_SCOPE_VIEW_LOG=1` prints both widgets' scale and fade, the focus and its share, and the
  ring's and smear's scales on each frame they change.

  Verified (`BF_CAPTURE_FPS=30`, `BF_MUTE=1 BF_BACKGROUND=1`):
  * The frame: Hawk with the Kman Auto at #60 take07's spot, `BF_MAP=e40 BF_CHARACTER=2
    BF_TEST_WEAPON="Kman Auto" BF_TEST_GOTO=0.29,166.87,-0.16,166.66 BF_CAMERA_PITCH=-0.02
    BF_NO_SQUAD=1 BF_TEST_SCOPE_PRESS=2.0,4.0`. Frame 90 beside take07 at 4.0 s: the rim, the
    rounded corners and the tick marks where the take has them. Against the same run with
    `BF_NO_HUD=1` (no frame), the picture comes out at 0.399 of itself outside the rim, 0.220 on
    its line, 0.764 on the ticks, 1.000 in the opening (the texture's 0.400, 0.216, 0.761, 1), the
    opening x 164.0-476.0 and y 127.3-352.7 units. The game's own surround (#60 take01, its first
    grow-out frame against the last whole one, the opening's own change taken out): 0.37-0.40. The
    frame is whole in the first scoped frame (frame 61; take01 26.267 s) and grows 1.2, 1.4, ...
    2.2 a frame on leaving (take01 31.28-31.47 s: 1.20, 1.40, 1.60, 1.82, 2.02, 2.22).
  * Flint's overlay: #126 take16's spot, `BF_MAP=e34 BF_CHARACTER=1 BF_TEST_WEAPON=L-Shot-75
    BF_TEST_GOTO=-52.96,30.96,-52.96,30.96 BF_TEST_SWEEP=164.4 BF_CAMERA_PITCH=-0.0199
    BF_NO_SQUAD=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_TEST_SCOPE_PRESS=2.0,5.0,8.0`.
    Frames 105 and 195 beside take16 at 3.5 and 6.5 s: the smear at the corners and the ring's
    dots in the same places; at level 2, the scene near black, the left and top edges read
    (5, 47, 48) and (4, 49, 58) against the take's (7, 49, 50) and (6, 51, 57), the corners
    (2, 21, 37) against (2, 20, 30). In: full in the 5th frame from the scope's first (the
    take's 6th); out: the corners and edges fall as the take's do frame by frame (the take's left
    edge 49 -> 24 -> 9 green, the demo's 47 -> 20 -> 9). #62 take06's spot on e40
    (`BF_TEST_GOTO=-16.16,152.62,-16.65,152.53`, presses 2.5 and 4.0): the same overlay over a
    lit cliff (the scene itself is the demo's, lighter than the take's).
  * The far plane by level: e10's follow camera (h_f727cfb9) gives 100 m (`BF_SCOPE_VIEW_LOG`).

  The others' aim wanders while scoped: a slow drift of up to about 0.7° that moves the view
  and the shots together. In the scope everyone can still move, but only at a walk. The
  crosshair moves to the middle of the screen at the same size, 64 units (see play.md, **Ammo,
  reload and the crosshair**). The eye stays
  0.25 m off whatever is in front (by the collision), so a close wall can't be looked through.

  Each level gone up plays the character's in sound and leaving the scope its out sound. They're
  stored beside its snipe-sound: `h_00dd1fe0` (the type's +0x260, which the press plays,
  FUN_00124340) and `h_02c94d34` (+0x264, FUN_001249b0's), about 0.3 s each. Flint has her own
  pair (`h_1fd08ea5`, `h_03e0936d`); the others share `h_135ec2ce`, `h_f9ffc972`.
* **Squad hand-over.** It plays one sound as the camera cuts: `h_07949f0d` (common bank,
  0.18 s), found in a capture at all five hand-overs. It replaced the weapon-switch pair used
  before.
* **The name label** (#88, play_hud.rs `LABEL_SHOW`): the select widget's label (FUN_000c92e0),
  the member's name in the heading font at 24 points (the record 0x3c7b90 on h_e0afcd52,
  `fonts::HEADING24`) centred on (320, 240), orange (FUN_000b87e0's ff a0 6a) at 0.8 alpha.
  Each selection shows it for 0.7 s (0xc9440: its timer 0.4 + 0.3): whole for 0.4 s from its
  first frame, then over the last 0.3 s its alpha falls to 0 and it swells to 1 + 3k^3 times
  (k the share of that 0.3 s gone: 4x at its end; 0xc9690). Control passes 0.5 s after the
  selection (the select widget's countdown, FUN_000c8760's +0x7c: `SELECT_TIME`, which was 0.6),
  so the swell runs across the cut. The recordings (`Friendly Fire 2 + Death Cam.mp4`) agree:
  "BRUTUS" from 22.867 s, whole and the same size from its first frame, fading from 23.267 s,
  the cut at 23.367 s; "HAWK" from 31.847 s (3.5 s after the death: FUN_000c9170's 3.5 s), fading
  from about 32.23 s, the cut at 32.333 s; their ink 272.0-368.7 x 230-251 for HAWK and
  256-385 for BRUTUS, the record's cells to a unit, and (209, 138, 86) over the gas. The
  recording's swell runs about 0.05 s ahead of the code's curve counted from the label's first
  visible frame (1.24, 1.56, 2.04 and 2.62 times HAWK's width at 32.330, 32.363, 32.397 and
  32.430 s; the curve gives 1.13 to 2.3 there): its select widget may show the label a frame or
  two after the selection (not traced). Not done: the blue label (0x4a5c90's 3c 69 dc) the
  game gives a member the select widget already holds. `BF_TEST_SELECT=<character>[,<s>]` picks
  that member at 1 s (or s), also together with `BF_TEST_GOTO`.
  Checked at 60 fps on sdm_e34 (`BF_CAPTURE_FPS=60`): `BF_TEST_SELECT=0,1.0`: "BRUTUS" whole from
  frame 61 (1.017 s; 256-385.3 across, (221, 136, 92)), fading from 1.433 s, swelling to 2.3x
  by 1.617 s; `BF_TEST_GOTO=15.3,37,15.0,29 BF_TEST_DIE=1`: "BRUTUS" from 4.517 s, fading from
  4.933 s, the cut at 5.0 s, swelling past it.
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
* **Liquids that burn** (#1, play_liquids.rs `burning_ahead`, `steps_into_burning`). A wingman
  doesn't walk into lava, sludge or toxic waste (a liquid whose character rate h_e0489ea0 is
  above 0, the one that hurts: levels.md, "What a liquid does to a character"): with ground in
  one ahead of it (its feet within 0.15 m of the surface or under it: where its calves' parts
  would be in, `PART_REACH`) it doesn't set off toward its place, so it waits at the shore, and
  the movement undoes any AI-steered character's step onto such ground (a wingman's, an enemy's,
  a cutscene's walk), as the ledge probe undoes a step off a drop; one already in may still step
  up and out. Water stays walkable (its rate is 0).
  * Why: the game's AI has no liquid code (no AI routine reads the liquid state or the liquid
    objects, todo/146-liquids leads §4.2); what keeps its squad out is its routes over the
    level's world points, and no harmful liquid has one over its floor on the SDM maps (sdm_e13's
    lava 0 of the 536 over it, sdm_m03's sludge 0 of 141, sdm_m07's lava 0 of 284; at most a
    couple ankle-deep at a shore), where water has many (sdm_e10's pond 87-272). The demo's AI
    doesn't walk those routes yet, so this stands in for them: a data-backed rule, not a traced
    one.
  * Not known: whether the game's squad follows its leader into lava when he walks in (a direct
    move to him isn't traced; #146's shot A4, not filmed), and how its routes take it round a
    liquid between a wingman and its place: the demo steers straight, so the wingman waits at
    the shore there (on sdm_e13 the squad starting across the lava from a leader moved by a test
    hook stays where it starts).
  * The level's hinters (#292, maps.md "Blockers and hinters") are walls to the AI's characters
    and not to the player's: sdm_e13's band round its outer lava shore holds the wingmen there
    as the game's AI is held, while the player walks through it into the lava. Its inner lava has
    no band; there this shore stop is what keeps them out.
  * Checked: `BF_MAP=sdm_e13 BF_CHARACTER=0 BF_START=9 BF_TEST_GOTO=-9.9,-111.3,-21.44,-105
    BF_TEST_GO_AT=4 BF_TEST_HEALTH=5000 BF_LIQUID_LOG=1` (`BF_VIEW_YAW=3.14159` to look back):
    Brutus walks into the lava, Tex and Hawk crouch at the shore and Flint stands on the rock
    behind them, 11-16 m from their places, and none of them takes a hit; `BF_LIQUID_LOG` prints
    "waits at a liquid that burns" twice a second for each.
* Hits come from the game's effect library (common/effects-common.ale, read with
  `tools/ale_tool.py`) and the flesh world material (21, WMAT_FLESH_HUMAN). Every hit plays its
  debris bundle: blood_puff (smokecard.tga mist), bloodsplat_s (diffuse_big.tga droplets in a
  jet, under a weak gravity field) and giblet (a blood.tga spatter). Each ammo type adds its own
  hit effect and sound: ballistic sparks (Flint, Hawk), the cutter's long white streaks with a
  pale blue flash light (Brutus) and the laser's orange embers with an orange flash (Tex). The
  emitter / appearance numbers in `play_fx.rs` are those nodes' parameters, read as Freelancer's
  ALE ones (Brute Force hashes their names). Each hit also leaves the character's hit decal on
  the ground past them (h_14410af1: the game's white splat textures tinted 128,34,34,220; about
  1 m across, as in the captures) for 40 s, fading over the last 20. Flint bleeds grey: she's
  synthetic flesh (world material 24, WMAT_FLESH_SYNTHETIC), which the data says through her
  parts' collision: the material most of her parts' triangles carry is 24, the other three's
  21 (#128: `Game::flesh`, read at spawn into `Loaded::flesh`; this was an inference from her
  grey decals before), whose debris bundle h_1f7ca034
  is blood_puff_b, bloodsplat_b and giblet_b - the same nodes in grey and blue-grey, slower
  and smaller - and whose hit sounds for the heavier ammo are metallic. Her hit decal
  (h_02d017b7) is tinted 50,50,50. Each hit and death splat is one of the type's hit decal list
  (h_fb4bfc68), the pool its second list (h_efc2a874); the squad's lists hold one decal each.
  (The Shrikes, flesh 23, lay theirs by the game's own rule and blend: [enemies.md](enemies.md),
  "Blood"; the squad keeps this reading until #150.)
  Verified with `BF_CHARACTER=1 BF_TEST_SUICIDE=1 -- --test`
  captures against Brutus (`BF_CHARACTER=0`): grey mist and splats for her, red for him; and
  (#128) Brutus killing her on the test map, frame for frame as before the flesh came from the
  data (character-types.md, "Verified"). A
  hard hit (15+ damage: L-Shot, Bower, a close grenade) can knock a character down: the body
  goes limp (the game's ragdoll, [ragdoll.md](ragdoll.md), thrown by the hit's push, harder at
  the top so it topples: the demo's start) for 0.7 s, thuds on the ground
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
  only runs for h_08c2d2ee = 0.12 s, so the texture sits at a fixed offset (0.446, 0.102).
  * **Drop** (measured from the game's memory and frames in `todo/43-memory-chip/`; the spawn
    code wasn't traced): on the frame the squadmate dies (one frame later in the demo when a
    grenade or gas kills them: those systems aren't ordered before the drop) (take02: the chip's first frame is the
    body's first reaction, full size, no build-up), 0.50 m in front of where they stood along
    their facing, its middle over their feet by Tex 1.098 m (standing and kneeling alike) and
    Hawk 0.925 m. Brutus and Flint weren't measured: theirs (1.016, 1.064) are their bind
    pose's lowest vertex under the model's origin, which is Hawk's within 2 mm but not Tex's
    (1.204), so a guess. It stays there (it doesn't follow the falling body), doesn't spin, bob
    or fall.
  * **Pickup:** any living squad member (the player or an AI one) whose feet come within
    sqrt(3) m (3D) of its middle takes it (take03 and staging: 1.34 m across with the chip
    1.098 m up, every time from three directions; whether the game tests 3D or across only
    isn't settled, and the test wasn't found in the code). It goes on that frame, h_ee2c16da
    plays and the pickup lines show "Memory Chip Recovered!" (no count). The footage's green
    "+ 2000" by the radar on the same frame is the score, which the demo doesn't keep; nor does
    it reclone the squadmate.
  * The green
  sprites and blue light the demo drew before were effect h_ee11d51f (powerup_pill +
  light_powerup_pill), which belongs to the DNA canister pickup (mesh h_e3e4caad, "Alien
  Technology Acquired!"), not to a death. (`BF_PICKUP_LOG=1` prints the chip's material, where
  each is dropped and when it's taken.)
  * Its pixel program (pixel shader 0, defined at 0x3ddf80 in default.xbe, set from
    FUN_0008fd20 through FUN_000a4d00) is one combiner stage, texture x constant c0 for colour and
    alpha alike, c0 being the h_e01baa40 colour with `alpha` as its fourth component
    (FUN_0008fa50 maps the names to +0x70 / +0x7c; FUN_000a4d80 packs them), then a final
    combiner that fogs the colour. No vertex colour, lighting or second texture. So it's drawn
    as Bevy's alpha blend of that, fogged, with its texture's own mip levels (five, 64 down to
    4: box averages, mean alpha 39, 49, 52, 43, 40 of 255; since #257 the format reader decodes
    them, where this module used to rebuild the chain). Without mips the one-texel traces break
    up a few metres off.
  * Not used: `time-scale` (h_01590d7a, 0.4) is mapped to +0x58, but none of the shader's own
    functions read it (and it couldn't move where the scroll stops). The wrapper's h_e59d69a0
    (60) goes to render state +0x294 of the state cache (FUN_0009e960, flagging the material
    when it isn't 255): STENCILREF (#4), the stencil value the chip's pixels write, which the
    game's glare reads as a luminance threshold (#289, levels.md "The glare"; see below).
  * **Not matched: the footage's fill.** In `todo/43-memory-chip/` it's a filled translucent
    green box with a darker circuit pattern, not a lattice: over dark sand (take02, background
    about 10, 8, 6) its middle is 19, 79, 28; over lit grey rock (take05 at 16 s, background
    about 75, 70, 58) 177, 253, 172 with a soft pale halo past its edges. A Light grenade
    burning 2.5 m away changes it by under 10% (take08 / take09). The second is above the
    background in every channel, red by 100, which no SRCALPHA / INVSRCALPHA blend of this
    material's colour (red at most 16) can do, and the fill between the traces needs alpha the
    texture doesn't have there (alpha 0 in two thirds of it at the top level). So what the code
    reads (FUN_0008fd20's blend states, pixel shader 0) and what xemu draws disagree; the demo
    draws what the code reads. **Explained by the glare (#289):** the game keeps every pixel
    whose stencil is 60 (the chip's) and whose luminance is above 60 / 255, blurs it and adds
    2.35 times it back (levels.md "The glare"). Over the dark sand the chip's 19, 79, 28 has a
    luminance of 55 / 255, under the key: nothing added, as take02 shows; over the lit rock its
    blend is above it, and the glare raises every channel and spreads the soft halo. The demo
    doesn't key the chip yet: the glare also multiplies by the frame buffer's alpha, which the
    chip's blend leaves at a value not worked out (a follow-up).
  * Earlier footage: far off (Friendly Fire 2, 33.8 s) the chip's inside is
    green +58 over the background, red and blue up too; the demo's is green +30 to +48 at 6-15 m
    on sdm_e34, red and blue a little down. Up close in front of a lit wall (DNA + Weapon
    Pickups, 12.4-12.6 s) the footage's is whitish cyan, red 165 over 60, with a soft halo.
    The pixel program can't raise red above the background (texture red 45 x 0.36) and the
    texture's mips don't fill it, so that light comes from something outside the chip's
    material: not found. In the DNA footage the Light grenades' beams are close by, and at
    9.4 s a soldier walking through the chip is lit the same whitish cyan. A guess, not drawn.
  * It sorts 1000 m nearer among see-through things than it is: the terrain's
    blended texture layers are drawn in the same pass, sorted by their chunk's middle, and
    painted over it on sdm_e34 (it all but vanished: a 7/255 difference). Known limit: it now
    sorts after every other blended thing (gas clouds, blood mist, liquids, see-through
    materials), so it shows crisp over a cloud between it and the camera. (The same value is
    also the pipeline's depth bias, about 1e-4 of the depth: it doesn't draw through walls.)
  * `BF_TEST_CHIP=<x>,<z>[,<s>]` drops a chip at (x, z) at that time (default 0.5 s), on the
    floor below 2 m over the player's middle, without a death.
  * Verified: Brutus killing Tex on the test map (`cargo run --bin bf_play -- --test` with
    `BF_CHARACTER=0 BF_TEST_GOTO=0,0,0,0 BF_TEST_KILL=3 BF_VIEW_YAW=0.7`): the chip appears on
    the frame Tex is first knocked back (none the frame before, as take02 #162 / #163), 0.50 m
    ahead of where he stood and 1.098 m over his feet, stays as he falls, and Hawk's AI walks
    into it and takes it 1.72 m away (3D), "Memory Chip Recovered!" on that frame; on sdm_e34
    standing
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
  frame, as in the recording (f1910 and f1911 are the same), as on a manual switch (**The name
  label**, #88). At exactly 4.0 s the squad switch cuts to that member's follow camera, and the
  name swells and fades across the cut. Keys 1-4 do nothing while you are dead. With nobody left to take
  over (the whole squad down (#41), or deathmatch) the follow camera stays on the body as
  before; on the test map the player then respawns where it stood 4 s after the death (#302,
  the demo's rule there: [testtools.md](testtools.md) "The squad").

  The hand-over, for a death or a manual switch, cuts to the camera straight behind the new
  character's facing, the whole body in view, as the recording's cut to Hawk does: the follow
  camera put at its place at once (#126: every character's camera is cut to its place each
  frame while no player controls it, so take21's hand-over to Brutus jumps without an ease;
  [camera.md](camera.md)). If the follow camera's wall test leaves less than its whole boom
  there, the heading swings in 10 degree steps to the nearest one either side that leaves it
  (the demo's choice: the game's hand-over with a wall behind isn't filmed). The new
  character's aim, fire, walking and squad-AI idling (the leader's heading it turned to,
  kneeling) are dropped, so the cut is to the plain follow camera with them standing still.
  Standing a kneeling member up is the demo's guess.

  After the cut the follow camera is the game's ([camera.md](camera.md)): turned toward a close
  wall its wall test steps it to 0.75, then 0.5 of its boom, then 0.4 m, and the body turns
  see-through as the camera comes close (#126, which took #89's walls in).

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
  - `BF_TEST_SELECT=0`: the manual switch's label (frame 16; whole from its first frame since
    #88), and frame 25 cuts to Brutus from behind.
  - `BF_TEST_GOTO=16.5,37,16.5,37 BF_TEST_DIE=0.7`, a cliff behind Brutus at the hand-over:
    - frame 71: the cut, from behind with his whole body in view;
    - from frame 72 the hook's autopilot (a `BF_TEST_GOTO` whose goal is its start) sets the
      camera heading to 0 every frame, which points it at the cliff, 1.8 m away;
    - the follow camera's clamp then pulled it in close to him (frames 73-89; before #126: its
      wall test now steps it in, and his body turns see-through). That comes from the hook
      and the walls, not the hand-over.
  - With `BF_DEATHMATCH=1` there is no death camera.
* `BF_COMBAT_LOG=1` prints hits (each with its factors: the hit part struck, the own side's
  x0.7, the difficulty step; play.md **Damage**), knockdowns and dives.
