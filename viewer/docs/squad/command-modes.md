*Part of [squad/](README.md), the former page `squad.md`.*

* **Squad commands: the command modes** (#318, the first slice of #33: `play_orders.rs`, the
  mode on `play_brain.rs`'s `Command`; todo/33-squad-commands leads §1-§6, its takes take01-13;
  every function in [code-map.md](../code-map/ai.md), "AI"). Each squad member's brain has a command
  mode (brain+0x58, the PCMD_ table 0x3bd210), which picks its personality and so what it does.
  The Squad Command menu (#319) gives the orders and says their line (#320, below);
  `BF_TEST_ORDER` gives them without the menu or its line.

  | Order (menu button) | Mode | Personality | Base goal | The demo |
  |---|---|---|---|---|
  | (none: the start) | 1 FOLLOW | `h_169b2725` | 75 WINGMAN_FOLLOW | the follow (**The follow**, above); combat allowed |
  | Cover Me (A) | 1 FOLLOW | `h_169b2725` | 75 WINGMAN_FOLLOW | the same follow, holding fire until an event allows it |
  | Fire At Will (B) | 4 WINGMAN | `h_ea20a17d` | 75 WINGMAN_FOLLOW | the follow at WINGMAN's distances; fires at what it sees |
  | Stand Ground (Y) | 3 GUARD | `h_155e52d6` | 2 IDLE_VARY | kneels where it stands, holds, fires from there once allowed |
  | Move To (X, then the cursor) | 5 MOVE_OUT | `h_0f69b8b2` | 72 MOVE_OUT | runs a route to the point (#335), then Stand Ground with e635ecc6 |

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
  * **Fire At Will** (WINGMAN) keeps the follow, closer: FUN_00020b10's WINGMAN set (8, 10, 7, 7,
    4 m against FOLLOW's 1.4 times it), no cover points (its chase holds within d2), and combat
    allowed for as long as the mode lasts. Its first follower would walk to a route point ahead of
    the leader (states 6-16, not done: **The follow**); here it follows as the others do and says
    the walk line.
  * **The squad's pace** (#345; play.md, "Ground speed"): an AI member moves at 0.8 x its type's
    speeds (FUN_0012cb40: 6.2-6.3 m/s running, 6.8-7.0 dashing), Move To and the follow within d1
    of the leader (14 m, 10 in WINGMAN) alike; the follow's run or dash beyond d1 is 1.25 x that,
    the type's own, 7.8-7.9 and 8.5-8.7 (the follow's speed record, FUN_00020b10, which
    FUN_00048270 hands to the character at any pace level above a walk). The player runs at the
    type's 7.8-7.9 and dashes at 8.5-8.7, so a member trails the player's dash and holds about 14
    m behind it, as #33 take04's did, and moves on at the boosted pace to catch up once farther.
    The pace level is the routes' (#335's drops and the ramp's climb back, 0.17 s a notch in
    FOLLOW and WINGMAN; #344's start from a stand, its step down after the move asked, the hard
    turn's walk, and the feelers' drops: a dash's hit takes the boost off for 1.5 s, the next drops
    it to a run for 1.5 s); not in the demo: the walk and the velocity ease (#350).
  * **Stand Ground** (GUARD): kneels 0.3 s after the order (a fit: take07 0.24 s, take08 0.52 s
    of game time), or stays down if it was kneeling already (the follow's crouch pause, or the
    controlled character left crouched), and stays, its facing kept, Brutus standing; no dodges,
    no stepping out of the
    leader's line of fire (the guard personality's entries for them are off). It fires from its
    spot at what it sees once combat is allowed.
  * **Move To** (MOVE_OUT): walks a route to the point (#335, [nav.md](../nav/squad-on-routes.md) "The squad on
    routes": GOAL_MOVE_OUT's start 0x45d00 routes to the cursor's point, its update FUN_00045fa0
    walks it at 2 m) at the dash, slowed near the route's points as FUN_0004d840 slows it (a run
    within 6 m of its end and a walk within 3.2 m; a walk within 3.2 m of a sharp corner, a notch
    down at a bent one: #344's second fix round), steered as the follow is (#344),
    and from a stand or after a detour at the pace its ramp allows, a notch a second (Move To isn't
    one of FUN_00058670's follow modes: a walk, the run after 1 s, the dash after 2; the game
    runs instead while the
    member stands in one of the level's +0x114 prisms, FUN_0004cb60: what those volumes are isn't
    traced, so the demo dashes; the leads' "a run with an enemy known" misread the test); at the
    route's end it's GUARD and asks e635ecc6 of the chatter gate (chatter.md; its block:
    probability 100, no replay delay, priority 6, a 3 s wait, heard at any distance). With no
    route it stands, still in MOVE_OUT. The route is the brain's one route record, so the AI
    update's stuck handling and its route round a friend in the way (FUN_00041870) are Move To's
    too; that route round, and the 7 s wait behind a player's character when there's none, only
    run inside the level's prisms, which the demo doesn't read (`BF_TEST_PRISM` stands every
    character in one): in open ground the members pass each other by the game's avoidance
    (take08's Hawk ran past Tex 1.5 m off without a pause; FUN_000255d0's push, #344, and the
    bodies' collision). Before #335 the walk was straight, bent round the controlled character
    (`pass_round`) and given up when it stalled (0.1 m in 1 s); the test map's floor, with no
    points to route over, keeps that straight walk and stall rule. A member with no point routes
    to its own point (FUN_00058b60): it walks there, to within 2 m, then it's GUARD.
  * **The hand-over** (FUN_0005cc20, FUN_000c8e80, FUN_00018230): handing control from a
    character more than 50 m from the one taken sets it to GUARD, and each other AI member in
    FOLLOW or WINGMAN to GUARD within 5 m of where it stands, else MOVE_OUT to there (the game's
    point is its world point). The one taken, if it was moving out, is set to GUARD. A
    character keeps its mode while a player controls it, for when control passes on; the one
    left has its allow flags set by that mode again (FUN_00018230's FUN_00018000), so left in
    FOLLOW it holds fire until an event allows it (take02: Tex a3 to a0 at the hand-over to
    Hawk, Hawk a3 to a0 when left in turn). A Molotov Mutant left **holding his lit bottle** has
    it put away as a press of V would (`skills::control_left`, from `released`, #211: the
    product owner's call, the demo's own), so the AI that plays him never lunges or bursts on
    its own ([his page](../skills/molotov-mutant.md)).
    **The lines** (#320, chatter.md "Lines of several parts"): the one left, if alive, says by
    its mode, through the chatter gate as control passes: Cover Me f415ab02 ("I'm right behind
    you"), Stand Ground 0e69bfe5, or e635ecc6 when the one taken is over 50 m away (the far case
    above has set it to Stand Ground), Fire At Will 15dc4e2d, and Move To, the other modes, nothing
    (`hand_over_tag`). In the far case the members it re-ordered answer, joined to that line:
    those set to Stand Ground e97ce7fd, those sent moving out ed05c0d7 for the first and e97ce7fd
    for the rest (the far case isn't filmed; #36 take13 and take14's were near). Replay delay 30 s
    (the blocks'), so tapping back and forth says it once.
  * **Toggle Ability** (#333; `play_orders.rs` `toggle_ability`, `toggle_goal`). The menu's White
    entry changes no mode: FUN_000c1280 pushes GOAL_TOGGLE_COMBAT_SKILL (127) on each selected
    member's goal stack (FUN_00024540, mode 2: on top and started at once; #35 take18: Hawk's goal
    15 went 127 at the order and back to 15 at the switch). The goal table (FUN_00012b40) gives
    goal 127 the goal holder's vtable +0x688 (start 0x36e80) and +0x690 (update 0x36f60):
    * **its start** keeps the skill's state (brain+0x3c4 bit 6) and, when the skill wants its
      character standing (the skill's +0x11c: Tex's and Flint's; Brutus's and Hawk's no), has the
      brain stand (0x3f0e0 with 0: the crouch wanted cleared). (Its +0x10c branch, for a skill
      that needs the brain's target, keeping its position at skill+0xc, is no squad skill's: it's
      the effect skills', #200, whose order the demo refuses, its brain giving no target point
      (`skills::attack_skill`): the Orbital Beam of a Confed Marine, the Air Strike of a Militia, in a test
      map's squad slot.)
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
  * **Not done.** The follow's route-point states (**The follow**): WINGMAN's members going on
    ahead of Tex and drifting 16-30 m off at rest (take05). The cover points that put a member of
    mode 4 or 5 in GUARD (FUN_00043860), the dodge odds by mode, Medkit's heal (#329), the
    prisms that open FUN_00041870's route round a friend in the way and its wait (#335: not
    read), the GUARD fight's own goal (ATTACK_GUARD_AND_COVER: the demo's fire from the spot), an event's goal
    pushed over Toggle Ability's while it waits (the demo's member stands its whole time).

  Test hooks: `BF_TEST_ORDER=<member>,<mode>[,<x>,<z>][@<s>][;...]` gives each order at its time
  (1 s without one) as the menu does: `<member>` 0-3 or brutus / flint / hawk / tex, `me`, or
  `all` (everyone but the controlled character); `<mode>` guard / stand, wingman / fire, follow /
  cover, moveout / move (x, z needed), or the PCMD_ number. `BF_ORDER_LOG=1` logs the mode
  changes, grants, their end and the arrivals; `=2` also ten rows a second of each member's mode,
  goal, allow flags (a grant's time left), place, distance to the controlled character and what
  it's doing (fire, dash / run / walk, aim: the gun up holding fire, kneel, stand), as the takes'
  watch logs. `BF_FOLLOW_LOG=1` (#331) logs each follow state change and why (with the leader's
  distance), each cover point claimed, the formation's lists when they change and WINGMAN's first
  asking the walk line; `BF_TEST_FORMATION=<member>,...` puts those members (in FOLLOW) in
  FOLLOW's list in that order at each level's start, where the list starts empty, to stage the
  column an order leaves (take06's: `flint,brutus,hawk`); BF_FOLLOW_LOG also says what a level's
  start dropped and each change of a member's leader, and logs the turn check; `=2` adds, every
  0.25 s, a row for each test map NPC set to follow (#364: its state, its distance to the
  controlled character, its place, its cover point). `BF_TEST_NO_HOSTILES=1` (triggers.md) keeps a level's enemies out, as the takes'
  snapshot had them. The test map's NPCs (`BF_TEST_NPC`) have no brain, so they never post
  EVT_SQUAD_IMA_TARGET: only the level's characters do (a staged NPC enemy is the takes' frozen
  one in that, whether it fights or not). **`BF_TEST_ORDER` stays silent**: it says no order line
  (the menu's, #320). The order lines are driven through `BF_TEST_COMMAND` below, the hook the
  ticket asked for as delivered: it presses the real menu, so the line is said by `give`, and
  `BF_CHATTER_LOG=1` shows its parts ([chatter.md](../chatter.md), "Test hooks").

  Verified (`BF_MUTE=1 BF_BACKGROUND=1`, captures at 15 fps; e34 staged as the takes, Tex at
  (-83.39, -82.79): `BF_MAP=e34 BF_TEST_GOTO=-83.39,-82.79,...,27 BF_TEST_NO_HOSTILES=1
  BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_ORDER_LOG=2`):
  * Stand Ground (take07: `BF_TEST_ORDER=hawk,guard@3`, Tex dashing east from 4.5 s): Hawk m3 g2
    a0 at 3.07 s, kneeling 0.4 s later (frames 56-68), held at (-82.8, -87.1) while Tex went
    43 m off.
  * Move To (take08: `BF_TEST_ORDER=hawk,move,-70.8,-79.5@8`): m5 g72 a0, GUARD 2.7 s after the
    order (a route since #335), at her dash (6.8 m/s since #345, her type's 8.5 x 0.8 as the game's
    AI moves: take08's plateau 6.84, its run 6.0-6.3; 7.2 before, the clip's root, and 2.9 s), 1.7 m
    short of the point (take08: 2.00 m across at its first GUARD row, 30.85 s, then 1.84 as she
    settles; the 1.9-2.0 once quoted was a later row, #348),
    her e635ecc6 sound 14c84c28 0.13 s later (take08's was 0fc11d92, the block's other), kneeling
    0.4 s after arriving (take08 0.52 s). From take08's own kneel, with Tex nearly on her way
    (`BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999
    "BF_TEST_ORDER=hawk,move,-89.5,-87.8@1;hawk,move,-70.8,-79.5@10"`): she stands, runs past Tex
    1.3 m off (take08 1.5 m) and arrives 3.3 s after the order, 1.9 m short (take08 4.2 s; 3.2 s
    with the clips' root; the velocity ease isn't in the demo). With #344's pace ramp from a stand
    (Move To's 1 s a notch) 3.86 s, 1.9 m short. With Tex right on her line (`BF_TEST_GOTO=-82.16,-85.05,...`): she
    passes him 0.8 m off at the closest in the 10 Hz rows (1.4 m with the clips' root) and arrives
    at the same time. (Before #318's third round she waited 7 s behind
    him and stood her ground: the wait-then-GUARD rule, dropped.) Through the chatter gate (after
    #155, `BF_CHATTER_LOG=1`): at the first point her e635ecc6 is said (variant 14c84c28, priority
    6, 7 m off); at the second, ordered at 10 s, a timed mute of e34's opening radio lines refuses
    it, as it refuses every line; ordered at 32 s, after them, she arrives 3.3 s later (3.2 with
    the clips' root) and it's said with take08's own sound, 0fc11d92 (the set's other variant:
    each is said before one repeats), the HUD's speech icon by her portrait for its 1.3 s
    (frames 527-547).
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
    the demo's FOLLOW kept 2-5 m off Tex and WINGMAN 1-4 m (means by member), at rest 5-6 m and
    4-5 m; the takes' are 12-16 m (FOLLOW) and 5-10 m (WINGMAN) running, about 9 m at rest in
    FOLLOW. Since #331, the game's follow (below).
  * **The follow** (#331), on the takes' staging (`BF_MAP=e34 BF_SCRIPT=off BF_TEST_NO_HOSTILES=1
    "BF_TEST_WARP=0,flint,-91.08,-83.87;0,brutus,-87.21,-84.16;0,hawk,-89.53,-87.82"`, Tex dashing
    to take04's stop: `BF_TEST_GOTO=-83.39,-82.79,-49.25,-71.14,27 BF_TEST_GO_AT=2
    BF_TEST_SPRINT=2`, `BF_ORDER_LOG=2 BF_FOLLOW_LOG=1`, 270 frames; the demo's 15 Hz rows against
    the takes' 10 Hz watch rows, the trail over the rows with Tex moving; main is `b3f6a9c`):
    * take04 (FOLLOW, no order): the three set off as Tex gets 11.2 m off each (Flint and Hawk at
      2.47 s, Brutus at 2.87 s; the take's 2.44-2.53 and 2.81 s); they trail his dash at a mean of
      13.0 (Brutus), 19.7 (Flint) and 18.6 m (Hawk), the take's 11.9, 16.2 and 13.3, Brutus at
      14.1-14.9 m once at speed (the take's 13.7-14.4); never ahead of him (main: 10-14 of the 33
      rows ahead, a mean of 4.9, 7.4 and 7.8 m); at rest 8.7, 9.2 and 10.6 m off him (the take's 8.5,
      9.1 and 8.9; main 6.3, 3.9 and 4.4). Brutus goes to the cover point h_e1182318, where the
      take's Brutus stood (0.3 m off it); Hawk claims h_f7d1227d, 118 deg off her dash, so the turn
      check walks her 1.47 s (the walk rows at 1.3-1.8 m/s, her distance to Tex closing 10.1 -> 8.7 m
      and opening to 9.9 m) before the run and the point; Flint, who claimed the same point before
      the claim test, holds where he stops (the take's Flint and Hawk ended 1.4 and 1.65 m off
      h_f7d1227d). The take's Brutus walked to his point (MT_WALK from gt 7.21, the frame Tex
      stopped, for 1.4 s, then the run at 8.71 and there at 8.89) where the demo's dashes: the
      demo's route to it starts at the point itself (his own point dropped, FUN_00041320), within
      75 deg of his dash, so the turn check doesn't fire; whether the game's route began at his own
      point, behind him, isn't settled (a live read of the pace record would). Flint's and Hawk's
      longer trail is their rise from a kneel, 1.1 s from the hold's end to a run (the stance's
      crouch2stand played whole) where the game's took 0.3 s (take04's motion 7); Brutus, who
      stands, matches.
    * take06 (Fire At Will at 4.3 s, Cover Me at 10 s, the dash at 14.5 s): FOLLOW's list Flint,
      Brutus, Hawk, so Hawk follows Flint: running 12.9 (Brutus), 15.5 (Flint) and 24.1 m (Hawk),
      the take's 12.4, 10.7 and 21; at rest 6.9, 9.2 and 18.3 m, the take's 7.2, 9.2 and 17.4.
    * take05 (Fire At Will at 4.3 s, the dash at 8.5 s): WINGMAN's list Flint, Brutus, Hawk;
      running 10.1 (Brutus), 13.1 (Flint) and 21.6 m (Hawk, following Flint), at rest 6.3, 6.6 and
      12.9 m. The take's Flint 5 m ahead of Tex running and 30 m off at rest, and its Brutus 17 m off
      at rest, are the route-point walk (not done); its Brutus ran 6.7 m and its Hawk 9.5 m behind
      Tex (Hawk 4.6 m at rest). Main's formation: 4.1-6.0 m running, 2.8-7.4 m at rest, 6-13 rows
      ahead of Tex.
    * The review's fix round (the build against the first round's, the same recipes):
      * a level after a level (`BF_MAP=sdm_e34 BF_TEST_NO_HOSTILES=1
        "BF_TEST_ORDER=all,fire@1;all,cover@2.5" BF_TEST_BACK=150 BF_MENU_PRESS=a@1.0
        BF_FOLLOW_LOG=1`: back to SELECT MISSION after 150 frames and sdm_e34 again): the second
        start drops the three entries of each list, and no member follows another there (no leader
        change in its 8 s); by the first round's code the lists stayed, so Hawk would have followed
        Flint from its first frame (the review's reading);
      * the order sequence on e34 (take04's staging with Tex standing,
        `"BF_TEST_ORDER=all,fire@1;all,cover@4;hawk,fire@7" BF_FOLLOW_LOG=1 BF_CHATTER_LOG=1`):
        WINGMAN's list Flint, Brutus, Hawk at 1 s, Hawk following Flint and Flint saying 1ecaeb98;
        FOLLOW's the same at 4.07 s, WINGMAN's kept; Hawk's Fire At Will at 7.07 s changes neither
        list, she still follows Flint and says nothing (by the first round's code she'd have been
        alone in WINGMAN's list: its first, following Tex, saying the line). A unit test runs the
        same. The bookkeeping now runs after the frame's orders and hand-over by the schedule
        (it was ambiguous with them, so a frame late or not by the schedule's luck: take05's and
        take06's Hawk, following Flint from the order's frame, holds and kneels 0.13 s sooner than
        the first round's, which chased Tex for that frame);
      * take04 as above, take05 and take06 unchanged (take06 at rest 6.9, 9.2 and 18.3 m); the e34
        run and sprint, #242 take01, e13rev, take08's Move To, m07_a, e34's Stand Ground fight, the
        deaths, the hand-over, the test map's floor and the squad's rows of the lava yard unchanged
        to the row;
      * the turn check elsewhere: the lava yard's Cogburn (98 deg, at his point 0.7 s later than
        before), the Move To hand-over's Tex (97 deg) and the doors' Tex (173 deg). There Tex's
        walk, slower, reaches the corner before the cover point later, and after it a step straight
        away from the point passes the walk's pass test (FUN_00025870 has no lower bound): his
        route ends 3.0 m from Brutus, 5 m short of the point (the first round's: on it, 7.6 m off);
        the same test ended the first round's Brutus 4 m short on the hand-over;
      * the claim test: e34's Fire At Will then Cover Me with its enemies sent Brutus and Flint to
        one point, h_e4690f7a (the first round's), Brutus alone now (on it, where Flint's crowding
        left him 0.7 m off), Flint holding 9 m off Tex; the fight the same (25 rows firing, no one
        dead). The Move To hand-over's Flint and Tex had both claimed h_09b3e901 (Tex never got
        there): Tex takes h_06073d1c now.
    * The walk line through the menu (`BF_TEST_COMMAND=30:hawk:0.4;30.9:brutus;31.5:flint;32.5:fire`,
      Tex standing, `BF_CHATTER_LOG=1`): Flint, WINGMAN's first, asks 1ecaeb98 at the first frame
      after the menu's fade, said alongside Tex's order line (take05: 0.58 s after the order's gate
      call); main asked nothing in the 3 s after (no formation walk set off).
    * Frames: from above (`BF_CAMERA_YAW=-1.5708 BF_CAMERA_PITCH=-1.0 BF_CAMERA_DISTANCE=22`) main's
      three run beside and ahead of Tex and gather 2-4 m from him; this follow's trail out of view
      behind him and stop 9-10 m back. The tutorial's lava yard (t335test's recipe) the same: main's
      marines stand in front of Tex at the yard's end, these behind him, out of the follow camera's
      view.
