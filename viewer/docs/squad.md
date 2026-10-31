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
  (EVT_GRENADE_NEAR -> GOAL_DIVE, the motion scripts' dive clip). This is the follow of the
  squad's default command mode, FOLLOW, and of Fire At Will's WINGMAN, closer; Stand Ground and
  Move To hold or go elsewhere (**Squad commands**, below, #318).
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
  so the swell runs across the cut. The recordings (`todo/reference/Friendly Fire 2 + Death Cam.mp4`) agree:
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
* **The squad's AI table.** A squadmate's personality (`common/ai/personality.xmb`) is its
  command mode's (#318, **Squad commands** below): FOLLOW's `h_169b2725` by default, WINGMAN's
  `h_ea20a17d` (each character's own `personality-wingman` is empty and inherits it) after Fire
  At Will, GUARD's `h_155e52d6` after Stand Ground, MOVE_OUT's `h_0f69b8b2` after Move To. It
  maps events to goals, with priorities and weights. Its events and goals are numbers. The
  names come from default.xbe's name tables: events at 0x3bdc14 (`EVT_DEAD` = 10, … 96), goals
  at 0x3bce78 (`GOAL_DODGE` = 59, … 134). Without enemies, these are the wingman personality's
  entries, which the demo's dodges use in every mode (FOLLOW's own have `GOAL_DODGE` 1 :
  `GOAL_NOOP` 3 for EVT_SHOT_AT and always `GOAL_DODGE` for EVT_DAMAGED_BY_PC; GUARD's take no
  dodge: not done):

  | Event | Goals (weight) | Here |
  |---|---|---|
  | `EVT_GRENADE_NEAR` (pri 28) | `GOAL_DIVE` | dive away from a grenade that lands near |
  | `EVT_SHOT_AT` (20.15) | `GOAL_DODGE` 1, `GOAL_NOOP` 7 | a shot of yours passing within 1.5 m (the distance is a guess): sidestep one time in eight |
  | `EVT_DAMAGED_BY_PC` (15) | `GOAL_DODGE` 1, `GOAL_NOOP` 2 | shot by you: sidestep one time in three |
  | `EVT_BASE` (1) | `GOAL_WINGMAN_FOLLOW` | the formation |
  | `EVT_IDLE_VARY_DATA` | `IDLE_VARY_DATA_PAUSE_CROUCH` 15, `…_PAUSE_CAUTIOUS` 12 (equal weights) | holding position, the wingman's (Fire At Will) draw: the first pause kneels (as in the captures), then kneeling and cautious pauses (standing, weapon ready, looking left and right) are drawn, lasting 15 and 12 s |

  The idle draw is the mode's personality's EVT_IDLE_VARY_DATA entry (#318): the table's row is
  the wingman's, not Stand Ground's. FOLLOW's (the default) adds `IDLE_VARY_DATA_PAUSE` (97, a
  plain standing pause) for 6 s, the three equally likely after the first crouch; GUARD's
  (Stand Ground) has `PAUSE_CROUCH` alone: it kneels and stays down. (Before #318 the squad drew
  the wingman's table in every case.) `BF_AI_LOG=1` prints their idle choices ("crouch",
  "cautious", "pause") and dodges.
* **Squad commands: the command modes** (#318, the first slice of #33: `play_orders.rs`, the
  mode on `play_brain.rs`'s `Command`; todo/33-squad-commands leads §1-§6, its takes take01-13;
  every function in [code-map.md](code-map.md), "AI"). Each squad member's brain has a command
  mode (brain+0x58, the PCMD_ table 0x3bd210), which picks its personality and so what it does.
  The Squad Command menu (#319) and its lines (#320) give the orders; `BF_TEST_ORDER` gives them
  without the menu.

  | Order (menu button) | Mode | Personality | Base goal | The demo |
  |---|---|---|---|---|
  | (none: the start) | 1 FOLLOW | `h_169b2725` | 75 WINGMAN_FOLLOW | the follow as before (`squad_ai`'s formation); combat allowed |
  | Cover Me (A) | 1 FOLLOW | `h_169b2725` | 75 WINGMAN_FOLLOW | the same follow, holding fire until an event allows it |
  | Fire At Will (B) | 4 WINGMAN | `h_ea20a17d` | 75 WINGMAN_FOLLOW | the formation 8 / 11.2 as wide; fires at what it sees |
  | Stand Ground (Y) | 3 GUARD | `h_155e52d6` | 2 IDLE_VARY | kneels where it stands, holds, fires from there once allowed |
  | Move To (X, then the cursor) | 5 MOVE_OUT | `h_0f69b8b2` | 72 MOVE_OUT | runs to the point, then Stand Ground with e635ecc6 |

  * **The start.** The brain's set-up (0x16ff0-0x17734) puts a squad type in FOLLOW and sets
    allow_combat and allow_cover itself: a fresh squad fights at once (every #33 take's fresh
    start reads mode 1, allow a3; take11's members killed a Shrike unordered), and Cover Me to it
    changes nothing (the menu skips a mode the member has, take11; Brutus kept a3 in take03).
  * **Changing mode** (FUN_00018040): the allow flags by the mode (FUN_00018000: WINGMAN sets
    both, FOLLOW, GUARD and MOVE_OUT clear both; take03 / take06: a0 after Cover Me from another
    mode), the goals dropped and the base goal drawn again: the member stands up (a kneel
    carries into GUARD's), its walk and pause start over.
  * **Allow combat.** Without it the attack check refuses to fire (FUN_00024f80 at 0x250c5) after
    raising the gun: the member turns to its target, gun up, and holds fire. An event whose entry
    in the mode's personality has the quick goal h_f0233972 allows combat for 10 s: being hit
    (EVT_DAMAGED), shot at (EVT_SHOT_AT, a hostile's shot within 1.5 m), being a hostile brain's
    target (EVT_SQUAD_IMA_TARGET: the brain's EVT_ENEMY_SEEN case, 0x3e45a, posts it to its
    target whenever it takes the event, so a member an enemy has picked out fights back before
    any shot; a frozen enemy, brain mode 6, posts none, as take12's) or a squad member firing
    within 20 m of it (EVT_SQUAD_MATE_FIRING, the type's h_e3d8aeef; the controlled character's
    shots count: FUN_0022f060), in FOLLOW, GUARD and MOVE_OUT alike (MOVE_OUT has no EVT_SHOT_AT
    entry: being shot at alone doesn't). Each of its own shots starts the 10 s again; when they
    run out, outside WINGMAN, combat is taken back (the AI update, 0x1b1ff). That holds for a
    fresh member too after its first fight (take11: a3 to a2 about 10 s after it).
    EVT_SQUAD_ENEMY_NEAR and FRIENDLY_SHOT_AT (45, 46) grant it in the data too, but their
    posters aren't found: not posted.
  * **Fire At Will** (WINGMAN) keeps the follow, closer: the formation's places and the
    distances it sets off (3 m) and dashes (8 m) at, times 8 / 11.2 (FUN_00020b10's set for
    WINGMAN against FOLLOW's 1.4 times it), and combat allowed for as long as the mode lasts.
  * **Stand Ground** (GUARD): kneels 0.3 s after the order (a fit: take07 0.24 s, take08 0.52 s
    of game time), or stays down if it was kneeling already (the follow's crouch pause, or the
    controlled character left crouched), and stays, its facing kept, Brutus standing; no dodges,
    no stepping out of the
    leader's line of fire (the guard personality's entries for them are off). It fires from its
    spot at what it sees once combat is allowed.
  * **Move To** (MOVE_OUT): runs straight to the point (#242 has the routes) at the dash, the
    last 2.5 m at a walk (the game runs instead while the member stands in one of the level's
    +0x114 prisms, FUN_0004cb60: what those volumes are isn't traced, so the demo dashes; the
    leads' "a run with an enemy known" misread the test); within 2 m of the point (FUN_00045fa0's
    walk at 2.0) it's GUARD and asks e635ecc6 of the chatter gate (chatter.md; its block:
    probability 100, no replay delay, priority 6, a 3 s wait, heard at any distance). A
    walk that stalls (0.1 m in 1 s, after its first second) ends the same way (a failed walk,
    0x46256); a liquid that burns stops it at the shore. The controlled character on its way is
    passed with 1.2 m of room (the way bends beside him until he's behind): the game routes round
    a squadmate in the way (FUN_00041870 -> FUN_000594a0) and only with no route waits, then
    goes GUARD silently after 7 s for a player's character; in open ground there's a route
    (take08's Hawk ran past Tex 1.5 m off without a pause), so the wait waits on #242's routes.
    A member with no point is at it: GUARD at once.
  * **The hand-over** (FUN_0005cc20, FUN_000c8e80, FUN_00018230): handing control from a
    character more than 50 m from the one taken sets it to GUARD, and each other AI member in
    FOLLOW or WINGMAN to GUARD within 5 m of where it stands, else MOVE_OUT to there (the game's
    point is its world point). The one taken, if it was moving out, is set to GUARD. A
    character keeps its mode while a player controls it, for when control passes on; the one
    left has its allow flags set by that mode again (FUN_00018230's FUN_00018000), so left in
    FOLLOW it holds fire until an event allows it (take02: Tex a3 to a0 at the hand-over to
    Hawk, Hawk a3 to a0 when left in turn). The line the one left says by its mode, and the
    answers, are #320's (the demo still says the follow line f415ab02).
  * **Toggle Ability** (#333; `play_orders.rs` `toggle_ability`, `toggle_skill`). The menu's White
    entry changes no mode: FUN_000c1280 pushes GOAL_TOGGLE_COMBAT_SKILL (127) on each selected
    member's goal stack (FUN_00024540, mode 2: on top and started at once; #35 take18: Hawk's goal
    15 went 127 at the order and back to 15 at the switch). The goal table (FUN_00012b40) gives
    goal 127 the goal holder's vtable +0x688 (start 0x36e80) and +0x690 (update 0x36f60):
    * **its start** keeps the skill's state (brain+0x3c4 bit 6) and, when the skill wants its
      character standing (the skill's +0x11c: Tex's and Flint's; Brutus's and Hawk's no), has the
      brain stand (0x3f0e0 with 0: the crouch wanted cleared). (Its +0x10c branch, for a skill
      that needs the brain's target, keeping its position at skill+0xc, is no squad skill's.)
    * **its update**, each frame it's on top: with a +0x11c skill and the member kneeling or
      crouch-walking (motion state 6 / 7) it waits; once the skill's state isn't the kept one it's
      done; else it sends CT_TOGGLE_SPECIAL, the command V sends (the character's +0x15c with 8),
      **every frame** until the skill has switched, so a refused toggle (Hawk's or Tex's clip
      under way, a reload, under 10% to switch on) is asked again. The goal machine runs only the
      top goal: the member stands meanwhile, firing nothing (one frame for Brutus and Flint, the
      clip's event for Tex 0.5 s and Hawk 0.83 s, longer if it must stand up or wait for its
      stamina). No time limit was found (inferred), so **a refusal that lasts holds the member
      where it stands** until an order changes its mode or the player takes it: Tex with no gun
      in his other weapon slot, or none with rounds after his top-up from the reserve; Flint
      with the clip in hand empty and nothing to reload it from. `BF_SKILL_LOG` gives the first
      refusal's reason, once a goal (`Skill::told`); the ones after go unlogged.
    * A member a test hook holds where it is (`BF_TEST_HOSTILE`'s, `BF_TEST_TARGET`'s, which skip
      the AI's goals) still runs the goal's command (`toggle_goal`), its standing left to the hook.
    * The AI doesn't run while the game is paused, so the first command comes as the menu's
      fade-out ends (take18: the order at 7.0 s, goal 127 from 7.09, Hawk stealthed at 8.30: the
      0.3 s fade, her clip's 0.83 s event).
    * The skill then runs under the AI as under the player (skills.md "The toggle"): the drain,
      off at 0 stamina, at death, by its own ends (Tex's second gun dry, Flint's clip dry); the
      skill sounds play only for the controlled character (FUN_0010e7a0: take18's AI Hawk is
      silent), the lines go through the chatter gate. No personality uses goal 127 (#35 leads §5):
      the AI never switches it again by itself, so it stays on till one of those or the next
      order. Its look is each skill's own (skills.md): Hawk fades out, Tex draws his second gun
      in his berserk motions, Flint with a rifle takes her sniper motions; **an AI Brutus shows
      nothing**: his on (0x13c020) sets the vision mode and then, only with a player's view on him
      (char+0x5e4, the camera FUN_0006ff00 attaches to the character it follows), the view's mode,
      his kind-0x100 material controllers (the shell h_1ce41323) and the red fade; without one
      it jumps past all three (0x13c038). So the orange view is the controlled Brutus's; ordered,
      his Vengar has its rules (1 hit point a second, x0.85 damage, the charge) and no look. Not
      filmed: no take has an AI Brutus in Vengar.
    * A mode change while it runs (another order, FUN_00039f00) or the player taking the member
      (FUN_00018150) pops it; the demo drops it at death too.
  * **The radar's command tabs** (#333; play.md "Radar", `play_hud.rs` `update_tabs`): each
    portrait's tab shows its member's mode, the controlled character's too: Cover Me's (FOLLOW)
    green arrows, Stand Ground's (GUARD) yellow bars, Fire At Will's (WINGMAN) ring of red and
    orange dots, Move To's (MOVE_OUT) blue arrow, each new one with a growing flash.
  * **Not done.** The game's follow itself (#331 for the squad, #243 for the marines:
    FUN_00020b10's state machine): in FOLLOW the
    takes' squad trails Tex's dash at 12-16 m and stops about 9 m behind him (take04), where the
    demo's formation keeps 1-5 m off him running and stops 4-11 m off, ahead of him (the five
    distances' roles are in the code map's FUN_00020b10 row); WINGMAN's members going on ahead
    of Tex and drifting 16-30 m off at rest (take05). The cover points that put a member of
    mode 4 or 5 in GUARD (FUN_00043860), the dodge odds by mode, Medkit's heal (#329), Move To's
    routes and its wait when there's none round a player's character in the way (#242), the
    GUARD fight's own goal (ATTACK_GUARD_AND_COVER: the demo's fire from the spot), an event's goal
    pushed over Toggle Ability's while it waits (the demo's member stands its whole time).

  Test hooks: `BF_TEST_ORDER=<member>,<mode>[,<x>,<z>][@<s>][;...]` gives each order at its time
  (1 s without one) as the menu does: `<member>` 0-3 or brutus / flint / hawk / tex, `me`, or
  `all` (everyone but the controlled character); `<mode>` guard / stand, wingman / fire, follow /
  cover, moveout / move (x, z needed), or the PCMD_ number. `BF_ORDER_LOG=1` logs the mode
  changes, grants, their end and the arrivals; `=2` also ten rows a second of each member's mode,
  goal, allow flags (a grant's time left), place, distance to the controlled character and what
  it's doing (fire, dash / run / walk, aim: the gun up holding fire, kneel, stand), as the takes'
  watch logs. `BF_TEST_NO_HOSTILES=1` (triggers.md) keeps a level's enemies out, as the takes'
  snapshot had them. The test map's NPCs (`BF_TEST_NPC`) have no brain, so they never post
  EVT_SQUAD_IMA_TARGET: only the level's characters do (a staged NPC enemy is the takes' frozen
  one in that, whether it fights or not).

  Verified (`BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps; e34 staged as the takes, Tex at
  (-83.39, -82.79): `BF_MAP=e34 BF_TEST_GOTO=-83.39,-82.79,...,27 BF_TEST_NO_HOSTILES=1
  BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_ORDER_LOG=2`):
  * Stand Ground (take07: `BF_TEST_ORDER=hawk,guard@3`, Tex dashing east from 4.5 s): Hawk m3 g2
    a0 at 3.07 s, kneeling 0.4 s later (frames 56-68), held at (-82.8, -87.1) while Tex went
    43 m off.
  * Move To (take08: `BF_TEST_ORDER=hawk,move,-70.8,-79.5@8`): m5 g72 a0, 15 m in 2.4 s at her
    dash (7.2 m/s; take08 6.0 run, 6.6 dash), GUARD 1.8 m short of the point (take08 1.9-2.0),
    her e635ecc6 sound 14c84c28 0.13 s later (take08's was 0fc11d92, the block's other), kneeling
    0.4 s after arriving (take08 0.52 s). From take08's own kneel, with Tex nearly on her way
    (`BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999
    "BF_TEST_ORDER=hawk,move,-89.5,-87.8@1;hawk,move,-70.8,-79.5@10"`): she stands, runs past Tex
    1.9 m off (take08 1.5 m) and arrives 3.1 s after the order (take08 4.2 s, her run slower).
    With Tex right on her line (`BF_TEST_GOTO=-82.16,-85.05,...`): she bends round him 1.5 m off
    (frame 176) and arrives at the same time. (Before #318's third round she waited 7 s behind
    him and stood her ground: the wait-then-GUARD rule, dropped.) Through the chatter gate (after
    #155, `BF_CHATTER_LOG=1`): at the first point her e635ecc6 is said (variant 14c84c28, priority
    6, 7 m off); at the second, ordered at 10 s, a timed mute of e34's opening radio lines refuses
    it, as it refuses every line; ordered at 32 s, after them, she arrives 3.1 s later and it's
    said with take08's own sound, 0fc11d92 (the set's other variant: each is said before one
    repeats), the HUD's speech icon by her portrait for its 1.3 s (frames 527-547).
  * Fire At Will (take10: `--test BF_TEST_ORDER=all,fire@1 "BF_TEST_NPC=3.0,shrike
    soldier,enemy,fight,-60,-79,,0" BF_TEST_NPC_HP=3.5,1,250 BF_TEST_INSTANT_KILL=0`, the Shrike
    facing away): the first hit 0.6 s after it appears, dead 2.5 s after (take10: the first
    member on it 0.2 s after, dead 4.5 s after).
  * Cover Me after Fire At Will (take12: `"BF_TEST_ORDER=all,fire@1;all,cover@2"`, the same
    Shrike, `BF_TEST_AIM_HOSTILE=14`): a3 to a0 at 2 s; from 3.3 s Brutus and Flint aim at it
    and fire nothing for 11 s; Tex's first shot at 14.1 s grants all three (a1) at 14.2 s
    (take12: 14.0 and 14.41 s), they fire from 14.5 s, it dies at 15.5 s, and the grant runs out
    at 25.5 s, 10 s after the last shot (take12: 29.06 s, 14.7 s after its grant). With the
    Shrike facing them (bearing 180) and Tex not firing: its shot past Brutus grants him at 4.07 s
    (EVT_SHOT_AT), his shots grant Flint and Hawk at 4.87 s (EVT_SQUAD_MATE_FIRING).
  * Cover Me from the start (take11: `BF_TEST_ORDER=all,cover@1`): "FOLLOW already: nothing
    changes (allow 3)", and they kill the Shrike at once.
  * A live enemy's target (e34 with its enemies, no `BF_TEST_NO_HOSTILES`:
    `BF_TEST_GOTO=-60,-79,-60,-79,30 BF_TEST_ORDER=all,guard@0.3 BF_BRAIN_LOG=1
    BF_COMBAT_LOG=1`, Tex 33 m off the squad): the Shrike Hound spawned beside them takes Hawk
    (on Stand Ground, a0) as its target at 0.67 s, and EVT_SQUAD_IMA_TARGET grants her combat
    at 0.80 s, before its first bite (1.07 s, which knocks her down); she fires from 2.67 s, her
    shots grant Brutus and Flint (EVT_SQUAD_MATE_FIRING, 2.60 s), and they kill it at 3.6 s.
    After #155's merge the same up to the bite, then her first shot and the others' grant at
    2.47 s and its death at 3.1 s: her hurt line no longer takes a draw from her random state
    (the chatter has its own), so her later draws differ.
  * Stand Ground to a member kneeling in the follow (`BF_TEST_GOTO=-83.39,-82.79,-87.2,-75.8,27
    BF_TEST_GO_AT=999 BF_TEST_ORDER=flint,guard@9`): Flint stays down through the order (frames
    126-160; the watch log "kneel" in every row), no stand-up and kneel again.
  * The hand-over's far case (`BF_TEST_GOTO=-83.39,-82.79,-25,-78,27 BF_TEST_SPRINT=4
    BF_TEST_ORDER=hawk,guard@2 BF_TEST_SELECT=2,14`): at 54 m Tex goes GUARD and kneels, Brutus
    (5.9 m off him) and Flint (11 m) move out to his spot and hold there. Taking Hawk while she
    moves out (`BF_TEST_ORDER=hawk,move,-50,-71@2 BF_TEST_SELECT=2,3.5`): MOVE_OUT -> GUARD at
    the hand-over.
  * Unchanged without an order: the tutorial's marines (brain.md's recipe,
    `BF_MAP=tutorial BF_TEST_GOTO=-15.6,-87.4,-60,-75 BF_TEST_GO_AT=14`, 420 frames) and e34's
    follow (take04's run, 255 frames) captured with main's build and this one (and again after
    the review's changes): no pixel more than 20 levels apart in any frame, the only differences
    inside the farcast's glow (particle noise). The follow's idle draw (FOLLOW's plain pause)
    comes after the first 15 s crouch.
  * The follow distances by mode (take05's run with Tex dashing, `BF_TEST_SPRINT=9`): running
    the demo's FOLLOW keeps 2-5 m off Tex and WINGMAN 1-4 m (means by member), at rest 5-6 m and
    4-5 m; the takes' are 12-16 m (FOLLOW) and 5-10 m (WINGMAN) running, about 9 m at rest in
    FOLLOW: the follow's own (#331), see **Not done**.
* **The Squad Command menu** (#319, the second slice of #33: `play_squad_menu.rs`; the radar's
  part in `play_hud.rs`, play.md "Radar"; todo/33-squad-commands leads §2 and §4, takes take01-03,
  take08, take09; every function in [code-map.md](code-map.md), "HUD"). How the player gives the
  orders above. The keys are play.md's (1-4 held, then I L K J B V; Esc or Backspace).
  * **Selecting** (the HUD's squad list, FUN_000c7cf0; its d-pad handler 0xc7e20). Each d-pad
    direction is a radar portrait's member: up Tex, right Flint, down Brutus, left Hawk (the
    type's slot, h_072ab4af; the demo's 4, 2, 1, 3). A press of a member that may be selected
    (FUN_000c8470: there, alive, in the world; the controlled character too, but not while the
    cursor is up) plays h_0c0d7e4d and flashes three chevrons at its portrait. Held 0.25 s it
    toggles the member's selection (h_fac5d6b4 on, h_0d93fbd9 off); let go before, it's a tap:
    with the menu closed it hands control over as 1-4 always did (the 0.5 s countdown, the name,
    on the release now rather than the press), with the menu open it toggles. Each action locks
    that direction out for 0.3 s. A selected member that dies drops out with h_0d93fbd9. The
    d-pad isn't read under another menu, during a hand-over, in a cutscene, or with no AI
    member in the squad (a deathmatch, the test map's squad of one: 0xc7f1b). While the menu is
    open the d-pad comes as presses only (the open menu locks the player's game input,
    FUN_000c11f0): a key still held from the hold that opened it does nothing more (take01:
    LEFT held 1.2 s, Hawk kept), and every press is a tap.
  * **The menu** (the squad command widget, FUN_000c0040 / FUN_000c0fb0). It opens as a member
    is selected: the game paused (single player; play.rs `PAUSE_SQUAD`), h_1bdc87d1, a 0.3 s
    fade in (FUN_000c0cf0). By k = (fade / 0.3)^2 (FUN_000c09a0) a backdrop multiplies the
    picture and the HUD under it by (1 - k) x 175 + 80 (80 / 255 open: the console's multiply,
    drawn as black at 1 - that^2.2 over Bevy's linear blend), the title and the rows come in at
    min(2k, 1), the six entries at k. The radar slides to the middle over the backdrop; the
    crosshair and the item box fade out (FUN_000c17c0). The layout (FUN_000c0180, FUN_000c0ae0,
    the table 0x3c8fa8): "Squad Command" (h_ece71b9d, the body font at 16, c0 d7 ff, a shadow)
    centred 30 above Stand Ground; the entries' 64 x 64 icons (h_e1cea7e9, h_19ed0ce8,
    h_f8ddf854, h_f822c386) in the face buttons' places round (362, 240) with their 10-point
    labels under them, Medkit and Toggle Ability's 32 x 32 Black and White discs (h_e6d67fa3,
    h_fe0c0f73) low right with their labels over them; "D-Pad Select Char", "[A B X Y Black
    White] Give Order" and "[BACK] Close Menu" at 12 with the button icon font at 12 (record
    0x3c5fe8 on h_0e233fad).
  * **The orders** (FUN_000c0d90): the first pressed of B Fire At Will, A Cover Me, Y Stand
    Ground, X Move To, Black Medkit, White Toggle Ability. Move To, Medkit and Toggle first drop
    the controlled character from the selection; with someone left, Move To brings up the
    cursor and the rest go to each selected member (FUN_000c1280: `play_orders.rs` `order` for
    the four modes; Toggle Ability pushes GOAL_TOGGLE_COMBAT_SKILL, which switches the skill as
    the game runs again, **Toggle Ability** above, #333; **Medkit** pushes the
    game's GOAL_HEAL, whose behaviour isn't traced (#329): the demo only logs it), with
    h_11d6f86a; the entry just given stays whole as the menu fades out (h_15262e1b) and the game
    runs again at the fade's end. BACK clears the selection: h_15262e1b and h_ffbe0bb9. Stand
    Ground, Fire At Will and Cover Me go to the controlled character's own brain too when it's
    selected (it keeps the mode for when control passes on). **The closing sound h_15262e1b**
    (here, at the cursor's end, and after an order) is the code's (pushed for FUN_000bb220 at
    0xc0f73, 0xc0fe7, 0xc113e and 0xbd962) but in none of the game's sound libraries (no
    sounds-*.xml names it, nor its bytes reversed; the other six, the press, on, off, open, order
    and emptied ones, are common's): the demo asks for it and hears nothing. Whether the game
    hears anything there isn't known: take01 has two onsets at BACK, 6.47 and 6.73 s, their ids
    not logged. **Medkit's button is the item box's Black too** (B, the demo's Black, play.md):
    held on past the menu's fade-out (0.3 s) it reaches the item box once the game runs again,
    which uses the player's own Medkit (or says "No need to heal"), as the game's held Black would
    once its input lock is let go (inferred). While the menu is open it takes the frame's presses
    from the keys, the mouse and a pad alike (the pad's BACK opens no objectives menu:
    campaign.md).
  * **The Move To cursor** (HUD+0x44, FUN_000bd860; the game runs meanwhile). Each frame
    (0xbdfb0) it takes the world point (`bf/level.rs` `WorldPoint`: every point element of the
    level: on e34 1015 world points and 271 others, cover, door, patrol points ...) nearest the
    crosshair's centre ray's point (`play_reticle.rs` `Reticle::point`) whose flags pass (+0x24
    bit 14 set: h_0bb771a2; bit 15 clear: h_1346678f; +0x28 bit 2, set on every point:
    FUN_00059b60) and that projects inside the 640 x 480 (FUN_0004d630, the filter 0xbdef0);
    with none (the view on the sky) the controlled character's place, and then A gives Cover Me
    (the +0xcc branch). Its marker, the object type h_18709e83's three-sided pyramid (mesh
    h_fa39b375, drawn as the level's objects are), stands on it turning a radian a second; "Look
    to select position", "[A] accept" and "[B] cancel" fade in at (191, 403-421) as the menu
    goes; the crosshair and the item box stay hidden. Accept (the left button, E, the pad's A)
    sends Move To with the point and hides it (h_11d6f86a, h_15262e1b); cancel (the right
    button, Escape, B) or an empty selection hides it (h_15262e1b), nothing ordered. A cutscene
    starting while it's up hides its rows and its marker, and no press reaches it meanwhile; it's
    back after, on the point the view then shows (the demo's choice: what the game's cursor does
    under a cutscene isn't traced). The menu's and the cursor's root is shown all along, as the
    item box's is, and goes
    with the rest of the HUD (the letterbox, the map editor's edit mode): the scope's frame and
    Flint's overlay draw only while every part of the HUD is shown (play_scope_view.rs).
  * **Not done**: a dead controlled character's press switching at once (0xc809c: the demo's
    death camera hands over by itself); split screen (the small layout 0x3c8fd8, no pause, the
    radar hidden while selecting); the tutorial's gates on the d-pad (FUN_000ba320 ids 11 and 14,
    #265); the order lines (#320); Medkit's heal (#329); the backdrop's and the texts' fades as
    the console's sRGB blend (drawn in linear light, a 0.3 s difference); what the cursor's
    input lock (FUN_000be240's FUN_001e7850) and the player state +0xa0 bits 0 and 2 stop while
    it's up: the demo takes the cursor's own keys only (E, Escape, the mouse buttons), WASD and
    the mouse look still move Tex.

  Test hooks: `BF_TEST_COMMAND="<s>:<what>[:<hold s>];..."` presses at each time, on the hooks'
  clock (the sim time plus the time the menu's pause held it): `<what>` a member (brutus /
  flint / hawk / tex or 0-3) holds its d-pad direction `<hold>` s (0.1, a tap, by default; 0.4
  selects); `stand` / `fire` / `cover` / `move` / `medkit` / `toggle` (or `y` / `b` / `a` / `x` /
  `black` / `white`) presses that order's button; `back`; `accept` / `cancel` the cursor's A and
  B. `BF_TEST_CURSOR_AT=<x>,<z>` takes the floor there as the cursor's ray point (aiming at it);
  `BF_TEST_SWEEP` (play.md) turns the view under the cursor instead. `BF_SQUAD_MENU_LOG=1` logs
  the selection (the takes' `sel` digits: game slots 0-4, Tex 1, Flint 2, Brutus 3, Hawk 4), the
  menu's opening and closing, each order, the cursor's point (its index, name, type and flags)
  and the accept.

  Verified (`BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps; e34 staged as the takes, Tex at
  (-83.39, -82.79) facing the take's Move To point: `BF_MAP=e34
  BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999 BF_TEST_NO_HOSTILES=1
  BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_SQUAD_MENU_LOG=1 BF_SOUND_LOG=1`):
  * take01 (`"BF_TEST_COMMAND=2:hawk:1.2;6.5:back"`): the press sound at 2.07 s, Hawk selected and
    the menu open at 2.20 (take01: 0.2 s after the press, as here), the sounds in its order;
    frame 60 against take01's 4.0 s: the title, the entries, their labels, the rows and glyphs at
    the take's places to a unit or two, the backdrop as dark (the picture and the bars), the radar
    over it at the middle with Hawk's face orange in its bracket, the crosshair and the item box
    gone; frames 40 and 95 identical (the world paused); the fade-out after BACK 0.33 s, the
    radar back in its corner, the world moving again.
  * take03 (`"...2:hawk:0.4;2.9:brutus;3.5:flint;4.1:brutus;5:stand;11:hawk:0.4;11.9:brutus;
    12.5:flint;13.5:cover"`): sel 00001, 00011, 00111, 00101 at 2.20, 3.07, 3.67, 4.27 (take03:
    2.34, 3.04, 3.64, 4.25); Stand Ground to Flint and Hawk (GUARD, a0), Brutus untouched; Cover
    Me to all three, Brutus "FOLLOW already", Flint and Hawk back to FOLLOW a0 (take03 alike);
    the three faces orange with their brackets at 3.9 s.
  * take02 (`"...2:hawk:0.1;5.5:tex:0.1;9:hawk:0.2;12:hawk:0.35;14.5:back"`): 100 and 200 ms
    presses hand over on the release (the name, the chevrons, the cut; Tex a3 -> a0 left), 350
    ms selects Hawk (controlled by then) and opens the menu, as take02.
  * take08 (`"...2:hawk:0.4;3:move;8:accept"`): the cursor on world point h_ef17a343 at (-67.8,
    28.1, -76.5), the point nearest take08's own first read; its rows on take08's to the pixel,
    the marker turning; A: Hawk MOVE_OUT to it. With `BF_TEST_CURSOR_AT=-70.8,-79.5`: take08's
    chosen point itself, h_ff113737 (flags ffff4000, as take08 read), Hawk arriving 1.9 m short
    (take08 1.9) and GUARD. With `BF_CAMERA_PITCH=1.0` (the sky): no point, the marker on Tex, A
    gives Cover Me.
  * take09 (`"...2:hawk:0.4;3:move;6:cancel"`): cancelled, the selection empty, nobody ordered;
    Toggle Ability (`9:hawk:0.4;10:toggle`) switches Hawk's stealth on as the game resumes;
    Medkit (`13:hawk:0.4;14:medkit`) logged only.
  * Toggle Ability through its goal (#333; skills.md "How it was checked" has the recipes):
    Brutus, Hawk and Flint on and off, Tex as an AI member with Brutus controlled, each command in
    the first frame after the fade-out; Flint stood up first; on sdm_e34 Tex ordered under 10%
    waited standing till 10% and switched on. The radar's tabs after each order: play.md "Radar",
    "The command tabs".
  * The keys and the pad (`cargo test --bin bf_play squad_menu`, `the_keys`, `the_pad`): 1-4
    held and pressed, the order keys in the game's order, Backspace; the open menu taking the
    frame's presses, the pad's too (I or the pad's BACK opens no objectives menu); the cursor's
    E, left button and the pad's A let go (no use, no fire). The live keyboard and a pad weren't
    driven (the runs are in the background, with no pad).
  * A cutscene under the cursor (`BF_MAP=m09_a BF_TEST_GOTO=-177.12,-223.64,-177.12,-223.64
    BF_TEST_NO_HOSTILES=1 BF_SCRIPT=m09_a_startup,6 BF_SCRIPT_LOG=1
    "BF_TEST_COMMAND=1:brutus:0.5;2:move"`): the map opens under its letterbox, so the menu and
    the cursor's rows are hidden with the HUD (the game's HUD alpha by the bars, which every
    top-level widget's draw takes: cutscenes.md) while the marker stands in the world (frame
    60); the script's 122 at 6.07 s hides the marker (the cursor's point, still taken along the
    cutscene camera's ray, isn't drawn in the middle of its shots: frames 108-405); its 123 at
    26.0 s brings the marker back and, as the bars go, the rows with the HUD (frames 407-419).
  * The rest of the HUD: the scope's pass (Flint's L-Shot-75 on e40's cliff, play.md's recipe
    without `--test`, `BF_SCOPE_VIEW_LOG=1`) fades its overlay in to 1.0, as on main; with the
    menu's root hidden while the menu is closed, as #319's first commit had it, it stayed at 0.
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
* Death: the death cry (chatter e1213623 - the captures' deaths match its lines; ed35ffae for a
  fall's), a thud, a
  limp body that stays on a blood pool (the character's other decal, h_16d327fd), blood
  splashed round where they fall (the hit's blood effects sprayed up, and five of their hit
  splats on the ground within 1.4 m: the demo's choice, the game's own death blood isn't
  known; Flint's decals are dark grey, 50,50,50, in the data: she's a synthetic), their memory
  chip beside where they fell (below). Every line goes through the chatter gate and its one
  queue ([chatter.md](chatter.md): the hurt lines, the death cry, the killer's line). The death
  cry's response picks the nearest teammate for its "X is down" reply ("Brutus is down!",
  "They've taken out Hawk."), whose variants need the alert value 0 where every squad member's
  is 10: so, as the code reads, no reply is heard (the demo said one half the time before #155;
  the xemu take A3 settles it). On the radar the portrait becomes the member's own skull (the characters'
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
