*Part of [squad/](README.md), the former page `squad.md`.*

* **The follow** (#331, `play_follow.rs`; every function in [code-map.md](../code-map/ai.md), "AI"): the
  goal of the squad's two follow modes, FOLLOW (the default, Cover Me) and WINGMAN (Fire At Will),
  is GOAL_WINGMAN_FOLLOW, whose update FUN_00020b10 runs a state machine at brain+0x6ac; the demo
  runs its formation states (the demo's own formation places, ahead of the leader, are gone):
  * **Its leader** is the controlled character, but for the third entry of a follow list, which
    follows the first (the formation record's tables 0x3ffc40 / 0x3ffc70 read through FUN_0005e540
    and FUN_0005e310, which count both lists whatever the member's mode). The lists start empty at
    each level's start (the old brains' destructor took the squad out of them); a member joins at
    each mode change and when the player lets it go (FUN_0005e8b0), in the order the orders reach
    the members (the squad list's slot order: Flint, Brutus, Hawk), the two follow modes not alike:
    Cover Me puts it in FOLLOW's list by its distance to the controlled character and leaves
    WINGMAN's as it is; Fire At Will puts it at WINGMAN's end and takes it out of FOLLOW's, unless
    it's in WINGMAN's already (then nothing changes); Stand Ground and Move To take it out of both.
    So after Cover Me or Fire At Will to all three, Hawk follows Flint: #33 take06's Hawk waited for
    Flint to be 13 m off while Tex was 27 m off, trailed Flint by 15-16 m and stopped 8.8 m from
    him, 17 m from Tex. After Fire At Will, Cover Me, then Fire At Will to Hawk alone she's still
    WINGMAN's third: she follows Flint and doesn't say the walk line.
  * **The five distances** (d0-d4) are 8, 10, 7, 7 and 4 m in WINGMAN and 1.4 times them in FOLLOW
    (11.2, 14, 9.8, 9.8, 5.6), measured to its leader in 3D.
  * **The hold** (states 2 and 3): its pace set to a stand for 0.3 s, then held while its leader is
    within d0 (1.5 x d0 standing on a cover point). Kneels and idles meanwhile, facing the squad's
    heading.
  * **The chase** (state 0): a route to its leader's own world point (FUN_000416a0 to his place;
    [nav.md](../nav/squad-on-routes.md) "The squad on routes"), walked at 2 m a point at the dash (its pace ramp climbing
    from where it stands), planned again while he's farther than d3 from the route's end; it stops
    within d2 of a leader who isn't running (FUN_00020ad0: his motion a stand or a walk): FOLLOW
    then looks for a **cover point** near him (state 4: the first `<cover-point>` his point's flood
    reaches within 12 m that lies within 6 m of the member, that no squad member stands on and that
    no nearer member in FOLLOW could take; walked to, state 5, and held at), WINGMAN holds. At the
    route's end it holds with him within d4, else routes again. So the squad trails a running
    leader in a column, never ahead of him, and stops about 9-10 m back (#33 take04: 9.1, 8.5, 8.9 m;
    Brutus took the cover point h_e1182318 there, as the demo's Brutus does).
  * **Its speed** (the follow's speed record, #345): 1.25 x the AI's 0.8 beyond d1, the type's own,
    0.8 inside it and on the way to a cover point; so a member chasing a dashing leader settles
    about 14 m behind him (take04: 13.7-14.5 m).
  * **The walk to a cover point** (state 5) starts with the turn check (FUN_00058740, which state 4
    runs on the route it plans): a cover point whose route starts more than about 75 deg off the
    member's facing makes it walk for 1.5 s before the pace ramp lifts it to a run (the analyst's
    reading of #33 take04's Brutus, MT_WALK from the frame Tex stopped for 1.4-1.5 s, then the run;
    in the demo's take04 it's Hawk who walks, see the verification below). A point another member
    has claimed is passed over (the search's claim test, on in FUN_00020170's filter), so two never
    head for one point.
  * **The test map's followers** (#364, [testtools.md](../testtools/npcs.md) "Follower"): an NPC set to follow
    runs this same machine as a member of FOLLOW does (play.rs `update_player` gives it to
    `orders::think`, with `follow::npc_follows` deciding that it follows: a friend of the controlled
    character, which isn't the editor's stand-in). It joins no formation list (`Command::joined`
    stays 0), so its leader is always the controlled character and so after a hand-over, as the
    tutorial's marines' is; it's one of the characters state 4's filter looks through (`mates`), so a
    cover point is never taken by two of the squad and the followers together, and it's let go
    cleanly (`follow::let_go`: its route dropped, its cover point given up) when it's set to stay.
  * **Not done**: the follow's route-point states (6-16), which only WINGMAN's first entry enters:
    FUN_0001adf0 gives it a point ahead of its leader along the level's progress chains, and it runs
    there (take05: Flint 5 m ahead of Tex's run, then 30 m off at rest; Brutus, then the list's
    first, 17 m off). The analyst's follow-up has traced them (the progress points' links, the
    leader's path record FUN_0005acb0, brain+0x718 / +0x71c = 10 / 16 m:
    todo/331-follow-states/wingman-leads.md), a ticket of its own; without them no member has a
    route point, and Fire At Will's three follow as FOLLOW's do, closer. Their one line, state 9's
    "on my way" (1ecaeb98), the first entry of WINGMAN's list says at its first follow update after
    the order (a stand-in, [chatter.md](../chatter.md) "The walk line"). Also not done: the flank
    point's walk (state 1, brain+0x738, FUN_00056980) and FUN_00020a80's 3 m round it, a leader near
    his brain+0x3c8 object, the cover point's facing, and the brake over the last 0.9 m (FUN_0002fba0,
    which FUN_00048270 hands on at a walk only, and the demo's walk moves by its clip's root).

  Around the follow, as before: squadmates step out of the lane you aim or fire down
  (EVT_SHOT_BLOCKED_BY_FRIEND: sideways, the demo's), sidestep now and then when you shoot them
  (EVT_DAMAGED_BY_PC -> GOAL_DODGE) and dive away from a grenade that lands near them
  (EVT_GRENADE_NEAR -> GOAL_DIVE, the motion scripts' dive clip); standing a while, Flint, Hawk and
  Tex kneel (stand2crouch -> rp_crouch_idle, crouch2stand to move on), Brutus stays on his feet.
  Stand Ground and Move To hold or go elsewhere (**Squad commands**, below, #318). **The routes**
  (#335, play_routes.rs, [nav.md](../nav/squad-on-routes.md) "The squad on routes") never chase straight, so the squad
  goes round a lava pit by its walkway rather than across it. **It steers through the AI's
  avoidance** (#344, play_avoid.rs, [nav.md](../nav/steering.md) "The steering", FUN_000255d0): three feelers
  along its heading turn it off walls, rocks and the AI's hinter bands (a metre off e13's lava
  walkway's, as #242 take01's members ran), a squadmate within 1.5 m ahead pushes it aside, else it
  heads straight for its next point, its move turning toward that at most 200 degrees a second
  (FUN_00019af0, #344's fix round: no zigzag), from the move it last commanded (#363). **It runs
  facing its way** (#363, [nav.md](../nav/steering.md) "The view and the motion"): its own view turns toward
  its way at 200 degrees a second (FUN_00019ed0) and its motion is picked from its move in that
  view (FUN_001195c0), never from the controlled character's camera: it runs, walks or dashes by
  its pace with its body on its move, steps sideways only for the moment of a sharp turn its view
  lags, and backpedals on a reversal (at 0.64 x its type's run) until its view has swung round.
  With a target it fights on the move: the gun on the target by the upper body as it runs, no
  shots while it dashes (the dash can't aim: its motion's aim="false", and the demo's gun is down
  on a sprint), and from where it stops. At a door on its way (a pair of door points) it
  stands half a second if the door is still shut, its feelers off for 1.5 s, then walks on through
  (the door step, FUN_00026820, #346); it never waits for a door. Stuck 2 s (no closer to its next
  point than its best) a member detours round it (FUN_000426b0, FUN_00042280), and the third time at
  one own point it's put on the point it was walking to (FUN_000425b0); a route the stuck handling
  ends, the chase holds at its end within d4 or routes again. Its pace (the pace ramp, FUN_00058670)
  starts from a stand and steps a notch every 0.17 s toward the pace it asks, up and down: a walk at
  once, a run, then the dash; a detour, a re-route, a hold, a hard turn or a feeler's hit sets it
  down for a while.
* **The test map's squad** (#302, play_testsquad.rs, [testtools.md](../testtools/the-squad.md) "The squad"):
  it starts with the player's character alone, and its squad menu (T) puts any character type in
  any of the four slots, or a whole predefined squad in all four (#339: the squad deathmatch
  squads, each member in the slot its type's h_072ab4af names, the radar's place). The slot (`Player::character`) is the place on the HUD, the key and the
  identity for the hand-over; what this page reads by a character's name (chatter, skills, the
  camera, Brutus not kneeling) goes by its type (`Player::data_slot`), the same in the main game.
  **A member of a squad deathmatch squad speaks** (#357): it says what the campaign four say (the
  Squad Command menu's order line, the hand-over's, the hurt, kill and death lines) from its own
  type's chatter library, `sdm_feral_warrior_chatter` ... `ty_mctavish_chatter`, with its type's face
  set moving its mouth where the sounds have viseme tracks; a tag its file lacks (the walk line, the
  "healed" lines ...) is refused as in the game, and McTavish, whose type has no name tag, is passed
  over in the order line's names ([chatter.md](../chatter.md), "The squad deathmatch squads' voices").
  They still have no skill (#144's and #200-#209's).
* **The tutorial's Confed marines** (#227, [brain.md](../brain/confed-marines.md)): the V-squad's two follow Tex
  with this follow (#331: they never change mode, so they join no list and follow him; #243, closed), see by their sensor (80 m, 90 degrees either way) and fire with the AI's aim
  (accuracy 0.6). In take11 they trailed Tex and stopped 9-10 m behind him, as the follow's d2
  stops them now. Switching to them (the d-pad) isn't done.
