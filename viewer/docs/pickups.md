# Health pickups

* **Health pickups** (`play_pickups.rs`). Placed `inventory-object`s whose item type (objecttypes
  `<inventory>`, `Game::items`) has function-type 5 or 19. Function-type is the game's IFSET_
  enum, named in default.xbe's table at 0x3be3cc: 5 IFSET_GENERIC_HEALING, 14 IFSET_AMMO_BOX,
  16 IFSET_MINIGUN, 19 IFSET_POWERUP_MEDKIT, 20-23 the power-ups. The power-ups (19 with an idle
  effect, 20, 21) are `play_powerups.rs`'s: "Power-ups" below.
  * Medkit (h_f5123ace): taken by walking over it (the walk-over's reach, below) into the
    squad's shared inventory, up to its stack-limit of 25, with its pickup-sound. When full,
    "<name> cannot pick up Medkit." shows and it stays.
  * Squad members take them too, running over one: a medkit into the same shared inventory (left
    alone when it's full, without the message), a Garo fruit for their own health (left alone
    at full health). `BF_TEST_TARGET=1` with a `BF_TEST_GOTO` start 8 m behind one stands the
    first squad member on it.
  * A placed one taken sends PICKUP_OBJECT (15) to the level's triggers that listen to it
    ([triggers.md](triggers.md), #220; none listens to a medkit in the levels played so far).
  * A zone starts with the Medkits its squad's placements carry, pooled with the rest of the
    squad's items (play_inventory.rs, #281; play.md "Items: the squad's starting inventory"): e34
    6, e40 20, m01_a 3, m09_a 10, the tutorial none. They're the inventory Medkit h_192d5337 (80),
    the one a placed Medkit gives, so a pickup adds to the same count.
  * The item box shows one item at a time, a grenade type, a Medkit, or another item of the
    squad's (the OrgSen, the Hat: #281), each the same way: its HUD icon (the type's own
    h_e5ec3f1f: Frag fe20b919, Medkit f647bbef, OrgSen 1b73c691) 64 units square, the name at the
    bottom, the count at the right from 2 up. Blue while the item can be used and red while it
    can't, with the icon grey: a Medkit at full health; the OrgSen or the Hat for Brutus, whose
    skill refuses their use types (its +0x1c, FUN_0013c160), where G says "Cannot use" (#196
    take26). A Medkit taken changes nothing in the box (#196 take34, #28 take08): NEW shows the
    first time the box or the list shows that stack, and only if no slot has shown it before. Tab
    steps to the next item; held, the item list opens round the box (up the next two, left the
    previous two) and the game pauses; G uses the selected item; B is the Black button's Medkit
    shortcut (the 80-point Medkit used, the item before it back on the release). All of it:
    [play.md](play.md), **The item box and the item list** (#196).
  * Medkits within 3.5 m of one another, on the same level, are one group. (The demo used to
    draw its own soft green glow over each group, fitted to todo/medkits glow.png; it was
    removed on 2026-10-19 because the game's own lights and placed effects are now drawn, #143
    and #258.) The group's medkits are laid out in an even grid
    round its middle (rows of ceil(sqrt(n)), a short last row centred), each turned like the
    first give or take up to ~11 degrees (fixed per medkit), spaced by the medkit's measured
    footprint plus 30% so none overlap (the png's four in
    two rows; the levels place them up to ~3 m apart). No ring: the blue ring is for gate panels only. The
    effect-objects levels place beside pickups (type h_117c1805, ALE powerup_spawn) aren't
    shown. The multiplayer maps' soft green glows where power-ups appear
    (powerup_spawn_location, h_04ad9a0e: 21 on sdm_e34) are placed effect objects, drawn since
    #258 (maps.md "Placed effect objects").
  * Using a medkit (G) plays the stance's use_item overlay (Sc_w1_/Sc_w2_use_item, ~1 s; events
    1a6b4920 reach, 0a6e8f79 in hand, 19f8311b used). The used medkit (the carried type's model,
    h_192d5337's archetype) with its cross turned red (its texture's blue texels, and its
    cross glow red), is in the throwing hand from 0a6e8f79. At 19f8311b it heals and plays the
    medkit sound, and the medkit falls from the hand, bounces and stays on the ground (the
    last 12 do).
  * The medkit's cross glows: the model's second piece is a flat quad over the painted cross
    with the untextured glow material h_031724e6 (shader h_f539fe8c, glow 0.10 0.34 0.83,
    wrapper opacity 50). Flat untextured glows like it are drawn at full strength (the
    opacity is for the see-through glass shells round other pickups), as in the png.
  * Pickups are loose: every inventory-object without an idle effect, and the used medkits.
    * **At the start** each sits in the level's pose, tilted to the ground under it (up to
      ~25 degrees; steeper is a step, so it lies flat). It rests on its lowest point, not its
      origin: a model's origin can be its middle.
    * **Kicks and blasts.** A character walking into one kicks it out ahead, faster than
      they're going and off to the side it was on, with a hop, rolling the way it's sent
      (once per 0.6 s, so it isn't pushed along). A grenade blast within 1.5x its radius
      throws it.
    * **Shots.** Anyone's shot whose line passes through one (its bounding sphere, before
      the shot's end) knocks the first it meets: 2.5 m/s along the shot, a 1 m/s hop, and a
      turn from how far off its middle it was hit.
    * **Tumbling** (`rigid_step`). It moves as a rigid body:
      * its mass is spread like its bounding box;
      * it touches the ground with its model's outermost points, about 26: the furthest
        vertex toward each of a box's faces, edges and corners;
      * each point that goes into the ground gets an impulse there: a bounce (0.3, from
        landings over 1 m/s) and friction (up to 0.6 of the bounce).

      Because the push lands on a point, not the middle, it turns the object too: it tips
      over edges, flips off corners and rolls to a stop. It takes 8 sub-steps per 1/15 s.
    * **Coming to rest.** Still for 0.25 s on the ground, it's at rest, unless it's only
      balanced on a point or an edge: the touching points' narrower spread is under 2 cm,
      like the Garo fruit on its point. Then it's pushed over the way it leans. At rest it
      eases flush onto the face it stopped on over about 0.25 s, keeping its heading,
      instead of snapping upright.
    * Moving ones push off the others.
    * The ground it settles, tumbles and rests on is a thing's: the level's invisible blockers'
      tops aren't floors to it (#298: only a character's body meets them; maps.md, "Blockers and
      hinters"). No pickup or lying gun of the 53 levels starts over one, so the level's own
      lie where they did; a thrown or dropped one no longer stops on one.
    * The bounce, friction, drag and thresholds are the demo's choices, not the game's.
  * The medkit sound is h_1538baad, the Sound whose file is h_15331992 (named "92193315":
    those bytes as stored in sounds-<level>.xmb), the same in all 54 levels' banks, 1.5 s.
    30% of the time the character also says a line of their "healed" chatter (line_tag
    12c35d69, whose block's h_ea21ae4b is 30), with the sound. In todo/medic + intenvory use
    case.mp4 Tex says e707f108 at his use and Brutus nothing at his. `BF_TEST_HEALED=1` always
    says the line.
  * Each pickup adds a line below the middle of the screen (FUN_000ad790, one of the HUD's
    three): the item's display name (h_1dd3a49d; its stringtable-name without one) alone for the
    first, "<n>x <name>" (the format at 0x40d734) once more of the same are taken while it's up;
    each pickup puts its time back to 4 s (the widget's +0x64, set by its constructor
    FUN_000ad690, #228; it was 3 s). take08's two medkits: "Medkit", then "2x Medkit" for 4.25 s
    after the second; take12's DNA line 295.4-300.0 s, fading out at its end. In the demo
    (`BF_TEST_GOTO=51,14,51,20`) Tex takes the tutorial's two at (50.6, 17.2) and (51.0, 17.5) on
    the same frame, so "2x Medkit" comes at once; the DNA canister's line (one) is its name alone.
    A pickup that took nothing, its stack full, reads "<name> (full)" in red (FUN_000ad790's
    count-0 form; a lying gun walked over, "Weapon pickups" below).
    Status messages ("No need to heal") show just
    above them. They stay put on the screen (the capture has them by the character).
  * What a medkit heals: an item's h_0a811e94 is the health it restores and h_11884f2e the
    stamina (STAMINA POWER has 0 / 50 where HEALTH POWER has 50 / 0), not a respawn time. The
    placed Medkit (60) gives, by its pickup-archetype, the inventory Medkit h_192d5337 (80):
    using one heals 80. What the placed one's own 60 is for isn't known.
  * Healing Garo Fruit (h_e711067e, function 19): eaten when walked over, +40, left alone at
    full health. HEALTH / STAMINA POWER are also function 19, the orbs with idle effects: see
    "Power-ups" below (#15).
  * **A taken pickup is gone for good** (#18; the medkits and fruit, the DNA canisters, the ammo
    boxes, the grenade packs, the OrgSen): FUN_0011c170 removes what it takes (FUN_000db3e0) and
    nothing re-adds it, so a campaign level's own medkit or box never comes back. It came back
    after 30 s before #18, a guess the code contradicts. Only a spawn-trigger's copy (every
    pickup of the multiplayer maps) is replaced, by its trigger's timer: see "Pickups from
    spawn-triggers" (#278).
  * **The walk-over's reach** (#18): every pickup is taken by the same FUN_0011c540 ->
    FUN_0011c170, every living member of a player's squad against it each frame, the AI's too:
    the pickup at most 1.8 m below and 1.0 m above the character's position (its pelvis) and
    dx^2 + dz^2 under 1.8 (1.34 m across): `powerups::in_reach` for all of them. The medkits and
    fruit used to take 1.0 m across and 1.5 up or down, the DNA canister 1.3 m.
  * Test hooks: `BF_PICKUP_LOG=1` lists the pickups and each one taken or used.
    `BF_TEST_HEALTH=<hp>` starts hurt (above the most, with the most raised to it: #152's lava
    runs at 5000, as #146's take03). `BF_TEST_MEDKIT=<s>` uses a medkit at that time. `BF_TEST_ITEM_LIST=1` holds Tab from the start (the item list open, the game paused).
    `BF_TEST_ITEM=<label>` selects a carried item at the start and `BF_TEST_ITEM_USE=<s>` presses
    G then (play.md "Items: the squad's starting inventory"). Verified (#281): e34's range spot
    with `BF_TEST_HEALTH=50 BF_TEST_ITEM=Medkit BF_TEST_MEDKIT=1.5`: "Medkit 6" blue, used,
    "Medkit 5" red at 115/115.
    `BF_TEST_CHIP=<x>,<z>[,<s>]` drops a memory chip there (see "Memory chip" in squad.md).

## DNA canisters (#228)

* An item of group-type 9 (IG_DNA_CANNISTER, the IG_ name table 0x3be594): the tutorial's Confed
  Marine DNA h_f4933c12 (stringtable-name "Alien Technology Acquired!", display name
  h_1dd3a49d "Confed Marine DNA unlocked!") in the base building at (-62.5, 15.6, 18.8). A squad
  member within the walk-over's reach (above, 1.34 m across) takes it. #228 fitted the reach to
  take12 at 1.3 m, where the line came between Tex's feet at 1.41 m (295.30 s, no line at
  295.25) and 1.25 m (295.44 s, the line fading in at 295.5); the code's 1.34 is between them
  (#18). Its pickup sound plays, the line shows its display name, and it's gone for good (an
  unlock is once).
* Its PICKUP_OBJECT starts the tutorial's timer467 (4 s), whose pulse plays Bratton's
  dxud_ai_00060 (4.22 s) ([triggers.md](triggers.md), "Object events").
* The end screen names it (`DnaFound`, play_endmenu.rs): the squad's inventory holds the
  canister in the game, and the end screen's DNA line (FUN_00108360) asks 0x83110 for the DNA
  found: the save's campaign/save-game/dna-archetype, else the first inventory item whose
  `<campaign-gameflag>` path (the canister's h_1d604469/characters/h_048b8580) is a game-options
  character's `gameflag-path` (h_f5a42d61's entries, FUN_00198b60), that entry's name (h_07f2f670:
  h_0ce3dafe "Confed Marine") then " DNA"; "No DNA Found" without one. The demo keeps the DNA
  canisters taken on the level and matches them the same way (`Game::option_characters`); the
  save's flag isn't kept (#27).
* What it unlocks (its gameflag-database 1: the global flag h_1d604469/characters/h_048b8580, the
  deathmatch's Confed Marine) isn't kept: the demo doesn't load the global flags.
* Verified (`BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_PICKUP_LOG=1 BF_TEST_GOTO=-58.9,21.26,-62.22,19.23
  "BF_TEST_PANEL=a,1;a,1;a,1;a,1"`, 120 frames at 15 fps): taken at 1.80 s, PICKUP_OBJECT ->
  timer467 -> sound463 at 5.80 s; "Confed Marine DNA unlocked!" under Tex from frame 27 to 87
  (4 s), as take12's line (its place on the screen: take12's is ~45 units lower and larger, a
  layout the demo's pickup lines don't have yet). With the objectives and the ending fired after
  it (`BF_MISSION_LOG=1 "BF_TRIGGER_FIRE=h_f0cac43d,10,2.5;h_065b02c7,10,2.6;h_065b02c7,10,2.7;h_065b02c7,10,2.8;h_f82e0ba6,ENTER_DETECTOR,4"`,
  380 frames): MISSION SUCCESSFUL at 14.07 s, "DNA Sequences Recovered" / "Confed Marine DNA"
  with the DNA icon at take12's place (frame 379 against take12 at 363 s).

## Weapon pickups (#17)

`play_weapon_pickups.rs`, written from default.xbe's code (the addresses below; the reading is
`todo/17-weapon-pickups/leads.md`'s) and checked against take11 of `todo/28-tutorial` (the
tutorial's L-Shot-50), `todo/61-l-shot-50` take04 (Tex), `todo/52-confed-lzr-50` take04 (Brutus)
and `todo/DNA + Weapon Pickups.mp4`.

* **What lies.** A gun is never taken by walking over it; X takes it. Lying guns are:
  * a level's weapon-object placements (object type 20 WEAPON, reticule-action 2 RACT_PICKUP),
    drawn as their own model in the placement's pose, no glow and no idle effect, with a full
    clip: the tutorial's L-Shot-50 by the crate at (-109.9, 17.3, -39.1) (placement h_f6a8a9b5),
    e34's iKhan-GPL at (98.6, 41.0, -49.4), the campaign's others (leads §1). A spawn-trigger's
    template isn't one: every multiplayer map's weapons, e34's Therm Sweeper, m02_a's second
    MK-ASLT and m02_c's iKhan-GPL lie as its copies, on their side under the trigger, from its
    spawns (#278: "Pickups from spawn-triggers");
  * the gun a swap drops, and a dead character's guns (the ragdoll's start, FUN_00127290, drops
    whatever both slots hold, FUN_0014dfd0; release_weapon's dead path, FUN_00117660 at
    0x118a64, does the same where a clip carries the event), with the rounds they had; they fall
    and tumble as the loose pickups do ("Pickups are loose" above).
  * a creature's weapons too, as the code has them: m04_a places Psionic Blast x2 and Breath
    Attack (class 7, ammo-count -1, the 7 mm dummy model h_ff18f12d) at (-207, 44.5, 76.8), and a
    dead Shrike Hound drops its Breath Attack. Nothing sets them apart: a placement whose
    h_1cb7d5f2 is false becomes a pickup whatever its class or model (0x14a0f7 -> FUN_0014a170),
    the dead's slot guns drop whatever they are (FUN_0014dfd0), the object update hands every
    weapon out of a holder to the use candidates (FUN_00157c40 -> FUN_000ba1c0), and neither the
    candidate loop (FUN_000bd180) nor the tracker (FUN_000bf1e0, FUN_000bf2f0) looks at a class,
    a model or the rounds. The squad's holders refuse class 7, so walking there shows "Tex cannot
    pick up Breath Attack." and the red ring over next to nothing. Not filmed (a shot at m04_a's
    spot and over a dead hound would settle it); where the game rests a model-less one (its
    collision-type is 0) isn't known: the demo puts it on the ground as every pickup, the
    placement has it 2 m up.
  * Taken, a gun is gone for good: nothing puts a placed campaign weapon back (leads §4.6), nor
    a medkit or a box (#18). A spawn-trigger's copy comes back as its trigger's next spawn
    (#278).
* **The vanish timers** (#278; the gun's +0x1a8, FUN_00157c40, `Vanish`): while one runs it counts
  down, and at 0 the gun is taken out of the world (the tick loop FUN_000d9840 -> FUN_000db3e0).
  Outside a mission (the multiplayer maps: the session's level kind isn't 0x17506391):
  * a gun a swap drops, 20 s (release_weapon, 0x1189e5);
  * a dead character's guns, 20 s (FUN_0014dfd0);
  * a gun a walk-over takes rounds from, 10 s, whether one ran or not (FUN_0011c170, 0x11c3fd).
  In a mission only a dead squad member's guns get one, 300 s (FUN_0014dfd0); the enemies' and a
  swap's dropped guns stay, as every campaign take has them. A use's start on the gun stops its
  timer (FUN_00121670). A level's placed gun and a spawn-trigger's copy have none.
* **The target** (the HUD's use tracker FUN_000bf2f0 and its caller 0xbf110): of the lying guns
  at most 1.7 m above or below the controlled character's position (the hips: its node) and 2.5 m
  away across (the target it has at 0.7 of its distance, so kept out to 3.6 m), in the front half
  of the aim (the dot of the aim's heading with the way to it at least 0; a target that isn't a
  pickup needs 0.5) and in its line of sight, the one most in front. None while scoped, dead or
  in a cutscene. A gate panel in reach keeps the prompt unless the gun is more in front.
* **The prompt** (play_hints.rs `draw_prompt`, FUN_000aa600's weapon case) at the hint line's
  place, as the panel's:
  * the character can take it when one of its two slots takes the gun's class (FUN_000aad20 ->
    FUN_000a9910 -> the slot holders' 0x14baa0; the classes are the character type's
    `limit-weapon`: Brutus 1 8 / 2 1, Flint 1 8 / 0 8, Hawk 0 8 / 1 8, Tex 2 1 / 2 1, with 0
    pistol, 1 rifle, 2 cannon, 8 none). Then "Hold [X] to swap L-Shot-50." (h_f1285f72, the gun's
    display name h_1dd3a49d), or for a gun with no magazine prop (h_0bfde1e8 "") and no ammo-regen
    "Hold [X] to swap iKhan-GPL. (16 shots)" (h_e456c40f, its rounds);
  * otherwise "Brutus cannot pick up Jax-iP." (h_0ff85d29), the words alone from the line's start.
  * So Brutus and Tex refuse the pistols (Foley, Jax-iP, LZR-10, Kman, MNR-7, Saryl-23), Hawk the
    cannons.
* **The ring** round the target (the ring widget, its update 0xa9bf0 and the reticle
  controller's 0xa9510; play_hud.rs): 48 units across once in, zooming in from 2.4 times that,
  fading in (its openness c up 1.2 a second, drawn 48 x (1 + 1.5 (1 - c^4)) at alpha c^4), and out
  the same way when there's no target; a new target takes it at once if it was in. Blue (23 53 d4)
  on one the character can use, red (a0 20 20) on one it can't, a lighter blue (3c 69 dc) while
  it's being picked up. The gate panels' ring is the same widget: 48 units, where it was 60.
  * It sits on the point the tracker keeps for its target (its +0x1c from FUN_000bf2f0):
    FUN_0015a480's, the target model's hardpoint h_10f81d34 if it has one (a gate panel's button:
    play.rs `PANEL_BUTTON`), else the target's position. None of the 59 weapon types' models or
    their parts has that hardpoint, so on a gun it's the gun's position, the middle of its model
    (the L-Shot-50's bounding centre h_e99750a9 is 1 cm from it). The reach and the line of sight
    are measured from FUN_0015a500's point, h_ee389721 or the position: the same for a gun.
  * It's projected through the view the frame draws (the camera's Transform as the cameras have
    just set it), not the GlobalTransform, which is the last frame's until PostUpdate: the ring
    was a frame behind its target whenever the view moved (#17's review). The camera shake
    (play_shake.rs) is added after and isn't followed.
* **The hold**: X (E) held 0.3 s on the target (the player controller's +0x50, 0x1332a0 ->
  FUN_000a9710), once per press. On a gun the character can't take the press only grumbles: its
  chatter line f2ce8278, one time in ten.
* **The use** (FUN_00121670, then the use update FUN_00120290):
  * the slot it goes to by the type's pickup mode (`limit-weapon` h_fb08d4c6, FUN_00121ac0):
    Flint and Hawk (1) the first slot that takes the class; Brutus (2) a cannon to slot 2, a
    rifle to the slot in hand; Tex (3) the slot in hand;
  * that slot switched into the hand first when it isn't there (the normal switch clip, both
    panels: #52 take04's Brutus over the LZR-50), then the drop-and-pickup overlay from the motion
    set (FUN_00121920's motion type: Tex's `Sc_w_drop_<hand>_pickup_<picked>` at its speed 1.1,
    Flint's and Hawk's `Sc_w1_drop_pickup_w1` / `Sc_w2_drop_pickup_w2` by the slot (Flint's at
    1.2), Brutus's `Sc_w1_drop_pickup_w1` for either slot), once no other overlay plays; a use
    whose clip can't start gives up after 5 s;
  * the clip is laid over the whole body, its root bone too (#293). Unlike the upper-body
    overlays (a reload or a use_item animates 36-44 bones and no root), the squad's pickup clips
    animate 51-68 bones, the legs and the root among them: it drops 0.42 m
    (Brutus), 0.50 (Hawk), 0.59 (Tex) and 0.64 (Flint) as the character squats, and moves up to
    0.3 m across over one planted foot (Tex's left foot steps out 0.38 m and back, Flint's steps
    back). The demo had laid every bone but the root, as for the other overlays: the legs folded
    under a body left at standing height, the soles up to 0.64 m off the ground (play.rs
    `apply_overlay`'s `root`). The takes bend the whole body down to the gun with the feet on the
    ground (#61 take04 6.6-7.1 s, #52 take04, take11 300.6-301.1 s);
  * while it plays the character is held where it stands: the stick neither moves nor turns it
    (FUN_001195c0 skips both while the playing overlay has h_f6a410e6, which every motion set's
    pickup overlays have: `Overlay::holds`, `GunUse::holds_still`). So the clip always plays
    standing, on every bone; before, a move during it laid the clip over the upper body only;
  * the gun is lowered, and there's no firing, aiming, reloading, switching, throwing or item use
    until the clip ends.
* **The clip's events** (FUN_00117660):
  * release_weapon: the gun in hand's rounds go into the squad's reserve of its ammo type as far
    as the stack takes them (FUN_0022b580, FUN_0014ce90) and it drops where the hand holds it,
    lying with the rest (take11's MK-ASLT, its 48 into the High ROF stack; on the test map,
    where the stacks start full, the gun keeps them). The weapon panel is empty until the grab
    and the crosshair hidden (take11 300.8-301.1 s); the switch's first sound (ff820fda);
  * grab_weapon: PICKUP_OBJECT from the lying gun to the level's triggers (the tutorial's router
    h_fa7e22af: "Click Right thumbstick to toggle sniper mode."), a new gun of its type in the
    slot and the hand with the lying gun's rounds, loaded at once from the reserve (FUN_0022daf0
    with 0), its name over its count on the HUD (play.md "The weapon's name after a pickup"), the
    switch's second sound (e1e97460), and the lying gun gone. The two sounds are the ones the
    takes have at the release and the grab (spectrograms: #52 take04 7.52 / 7.95 s, #61 take04
    6.57 / 6.88 s, take11 300.73 s). The new gun is held in the stance of its class for Brutus
    and Tex, of its slot for Flint and Hawk (play.md's table, FUN_0011aa20), and stowed where
    its slot's holder hangs guns: for Tex his left ramp, whichever slot (#297: a rifle Tex took
    in place of his Minigun had been held and stowed as the Minigun).
  * The prompt for the next gun is back from the grab, while the clip plays on.
* **Walking over a lying gun** (FUN_0011c540 -> FUN_0011c170), any living squad member, the AI's
  too: within 1.8 m below and 1.0 m above its position and under 1.34 m across, a gun with rounds
  whose ammo type a squad member's gun uses gives them to the reserve, keeping what doesn't fit,
  with the ammo item's pickup sound, PICKUP_OBJECT from it and the line "2x Sonic Ammo"; with the
  stack full nothing goes and the line reads "Sonic Ammo (full)" in red (a0 20 20), every frame
  it's walked over, on the HUD of the player who controls the walker alone (an AI squadmate on
  a full stack shows nothing: FUN_0011c6a0 with AL clear, #18) (the DNA video's "Bio Ammo
  (full)" and "2x Sonic Ammo"). In #61 and #52's takes nobody alive used the dropped guns'
  ammo: no line, as in the demo.
  * The guns that count are what each member's slots hold now (FUN_00122ed0 through
    FUN_0014dc50, which gives the hand holder for the slot in hand): between a release and its
    grab that slot holds nothing (`HeldWeapon::empty`), so the gun just dropped doesn't count by
    itself (no "(full)" line over it every frame with its stack full).
  * It runs before the clips' events: on a grab's frame the lying gun is still there for it, but
    the new gun's ammo type wasn't counted yet; a gun a pickup has grabbed is gone for it (#17's
    review: the other way round, the grab's own gun was walked over the frame it was taken, its
    rounds into the reserve a second time, "6 / 16" for "6 / 10").
* **The AI never swaps guns**: no goal of the game's picks one up (leads §9).
* **Not done, or not the game's yet:**
  * where a dropped gun lands: it falls from the hand with a small spin (the demo's; the game's
    drop, FUN_0014a1d0's physics, isn't read: shot list C);
  * the vanish timers aren't filmed (shot list C2): they're the code's (above);
  * the use's walk to the gun (FUN_00120290's phase 1, when the gun is beyond 2-3 m of a slot in
    hand or 1 m otherwise): the demo uses it from where the character stands;
  * take11's view turns about 20 degrees during the use (300.6-301.2 s, no stick input): not
    traced, not done;
  * the faint grey shape over the weapon panel between the release and the grab;
  * the game's slot left empty by a use cut short after the release: the demo takes the gun at
    once instead; a character with fewer than two guns doesn't pick into the empty slot;
  * BEFORE_USE_OBJECT (18) at the use's phase 2 isn't sent (nothing the levels have listens);
  * the times against the takes (the tester's measures, #17's review): the demo's release and
    grab come 0.1-0.2 s early, and Brutus's switch before his pickup (#52 take04) lasts about
    0.2-0.3 s longer than the take's. The clip rates are the data's (`speed`; the type's rate and
    the skill's factor, FUN_00150290's, are taken as 1), the hold the code's 0.3 s;
  * the shot list's open questions (leads §0): a part-used lying clip (the code reloads it at
    once at the grab), the hold time's edge, a swap while scoped;
  * a pickup kneeling: the game plays the clip's `_cr` version there (FUN_0014fd50 takes the
    crouch's suffixes in motion states 6 and 7), which all four have (Tex's
    `Sc_w_drop_rifle_pickup_rifle_cr` 1.17 s, Brutus's `Sc_w1_drop_pickup_w1_cr` 1.00 s, Flint's
    and Hawk's `Sc_w1_drop_pickup_w1_cr` / `Sc_w2_drop_pickup_w2_cr` 3.0-3.17 s, all whole-body);
    the demo plays the standing clip over the kneel's upper body, as its other overlays (not
    filmed).
* **Test hooks:** `BF_TEST_GUN=<weapon>[,<ahead m>[,<right m>[,<rounds>]]][;...]` lays those
  guns round the controlled character's start, 3.5 m ahead by default (the takes' carrier was put
  there and killed), full unless given rounds; `BF_TEST_SWAP=<s>[,<s>...]` holds X 0.5 s from
  each time (`BF_TEST_USE=<s>` holds it 1 s); `BF_WEAPON_LOG=1` prints the lying guns, the
  target as it changes (its class, the slots' classes and the mode), each use, its clip, the
  release and the grab with the rounds and the reserve, the walk-overs and the dead's drops (with
  their vanish timers), each gun that vanishes, and the spawn-triggers' gun copies, and (play.md)
  where each gun hangs whenever that changes, with its class and stance;
  `BF_FEET_LOG=1` (play.md) prints the soles' height every frame with the clip's time.
* **Verified:**
  * the tutorial (`BF_MAP=tutorial BF_TEST_GOTO=-106.5,-43.5,-108.6,-40.6 BF_TEST_SWAP=3.0
    BF_WEAPON_LOG=1 BF_HINT_LOG=1`, 105 frames) against take11 299.5-301.4 s: "Hold [X] to swap
    L-Shot-50." where the take has it, the 48-unit blue ring on the gun, zooming in as the take's
    (291.8-292.4 s); X held 0.3 s, `Sc_w_drop_rifle_pickup_rifle` at x1.1, the lighter ring and no
    prompt; the MK-ASLT released at 0.25 s (its 48 rounds to the High ROF stack, 500 -> 548), the
    panel empty and the crosshair gone; the grab at 0.55 s: "L-Shot-50 6 / 0" name over count,
    the hint "Click Right thumbstick to toggle sniper mode." (blinking), "Hold [X] to swap
    MK-ASLT." 40 lower under it, the ring on the MK-ASLT at Tex's feet;
  * #61 take04 on the test map (`--test BF_TEST_GOTO=0,0,0,-3.0 BF_TEST_GUN="L-Shot-50,3.6,0.3;
    Jax-iP,3.3,-0.35" BF_TEST_WEAPON="MK-ASLT;Confed LZR-50" BF_TEST_SWAP=5.9`): "Tex cannot pick
    up Jax-iP." walking in, then "Hold [X] to swap L-Shot-50."; X at 5.9 s, the grab at 6.87
    (the take's lying gun gone between 6.75 and 6.85), "L-Shot-50 6 / 50", then the Jax-iP's red
    ring and refusal, as the take;
  * #52 take04 (`--test BF_CHARACTER=0 BF_TEST_GUN="Confed LZR-50,3.5,0.25;MK-ASLT,3.3,-0.4"
    BF_TEST_WEAPON="Feral Cutter;Bower 20" BF_TEST_SWAP=5.9`): the switch to slot 2 (both
    panels), `Sc_w1_drop_pickup_w1`, the Bower 20 dropped, the grab at 7.87 s (the take's
    7.85-7.95), "Confed LZR-50 99 Regen" over "Feral Cutter", "Hold [X] to swap Bower 20.";
  * Flint over an LZR-10 (`BF_CHARACTER=1`, L-Shot-50 + Jax-iP): the switch to slot 2, then
    `Sc_w2_drop_pickup_w2` at x1.2, the LZR-10 in place of the Jax-iP, the L-Shot-50 kept;
  * the DNA video's lines (`BF_CHARACTER=0 BF_TEST_WEAPON="Jax-iR;Feral Cutter"
    BF_TEST_GUN="Jax-iP,3.0,-0.3,2;A50 Bioreactive,3.6,0.4" BF_TEST_RESERVE=50`): "Brutus cannot
    pick up Jax-iP." with the red ring and "2x Sonic Ammo" (24 / 52); the iKhan-GPL's "(16 shots)";
  * e34's placed iKhan-GPL is found and offered; a squadmate killed (`BF_TEST_KILL=1`) lets go
    of both guns with their rounds;
  * no regressions: the tutorial gate's panel (hints.md's command): its prompt, its ring (now
    48 units, zooming in), its use and the gate.
  * after the review: #61's recreation with `BF_TEST_RESERVE=10` reads "L-Shot-50 6 / 10" from
    the grab (Tex walks over the gun on the way in and takes its 6 into the 11mm stack, 10 -> 16,
    "6x 11mm Ammo", since a gun in the test map's squad uses that ammo; the grab's reload takes
    them back); the tutorial run as above, the same times and counts; m04_a (`BF_MAP=m04_a
    BF_TEST_GOTO=-203.5,78.5,-205.4,77.6`): "Tex cannot pick up Breath Attack." and the red ring
    on the creature weapons, as the code reads.
  * #293, the whole body: the #61 and #52 recreations above and Flint's and Hawk's
    (`BF_CHARACTER=1 BF_TEST_GUN="Confed LZR-10,3.5,0.25" BF_TEST_WEAPON="L-Shot-50;Jax-iP"`,
    `BF_CHARACTER=2 BF_TEST_GUN="Saryl-23,3.5,0.25" BF_TEST_WEAPON="Jax-iP;MK-ASLT"`, each with
    `BF_TEST_GOTO=0,0,0,-3.0 BF_TEST_SWAP=5.9 BF_FEET_LOG=1 BF_VIEW_YAW=1.4`, 125-135 frames):
    each squats to the gun with its feet on the ground, on every frame of the clip the soles
    within 3 cm of it (Tex -0.027 to +0.001 m, Flint -0.026 to +0.002, Hawk -0.016 to 0, Brutus
    0, his hand dipping 6 cm under the floor's plane at the grab), where Tex's had been up to
    0.58 m over it; the switch before Brutus's and Flint's pickup as before. With the stick
    pushed from 6.4 s (`BF_TEST_GOTO=0,0,0,-8 BF_TEST_GO_AT=6.4 BF_TEST_GUN="L-Shot-50,1.6,0.3"`)
    Tex stays where he stands through the clip (6.27-7.33 s) and sets off after it.
  * #299, a swap moves rounds and makes none: the release keeps in the gun what the stack doesn't
    take (0x118a09-0x118a22: FUN_0014ce90 returns how many went in, the gun's +0x1f8 less them)
    and the grab takes the lying clip and what it lacks from the reserve (`release_rounds`,
    `grab_rounds`, the unit test `the_swap_moves_rounds`). Eight swaps back and forth with
    `BF_TEST_SWAP=3.0,6.0,...,24.0 BF_WEAPON_LOG=1`, the clip in hand, the lying gun's and the
    reserve adding up the same after each: the tutorial's L-Shot-50 and MK-ASLT (the command
    above: 548 of High ROF, 6 of 11mm); the test map's laid L-Shot-50 (`--test
    BF_TEST_GOTO=0,0,0,-0.5 BF_TEST_WEAPON="MK-ASLT;Confed LZR-50" BF_TEST_GUN="L-Shot-50,1.6,0.3"
    BF_TEST_RESERVE=10`: 58 and 16), and two MK-ASLTs with shots between (`BF_TEST_GUN="MK-ASLT,
    1.6,0.3,20" BF_TEST_RESERVE=30 BF_TEST_TRIGGER=1.0-1.6,7.5-8.2,13.5-14.0`: 94, less each
    burst); e34's placed iKhan-GPL against Tex's Jax-iC (`BF_MAP=e34
    BF_TEST_GOTO=98.6,-47.0,98.6,-48.6`: the Jax-iC keeps its 36 with the Sonic stack full, the
    iKhan its 16, its ammo-type 9 having no ammo item); a dead squad member's guns
    (`--test BF_TEST_GOTO=0,0,0,0 BF_TEST_KILL=1 BF_TEST_SWAP=4.0,7.0,...`: Flint's L-Shot-50,
    walked over first, its 6 into the reserve and back at the grab); sdm_e34's Saryl-45 copy
    (the gun recipe in triggers.md, swaps every 3 s: 30 and 50 throughout, then the copy's
    respawn at 22.85 s lays a new full one, as it should). The test map's rack, the copies'
    respawns and the LZRs' recharge are the sources of new rounds.

## Ammo boxes (#18)

`play_item_pickups.rs`, from default.xbe's walk-over (FUN_0011c540 -> FUN_0011c170's item path;
the reading is `todo/18-ammo-pickups/leads.md`'s, its addresses are in
[code-map.md](code-map.md)) and checked against that folder's takes 01-03 and 08 (sdm_e34, xemu,
Tex's half of the split screen).

* **What they are.** Ten ammo items (objecttypes `<inventory>`, function-type 14 IFSET_AMMO_BOX,
  group-type 1 IG_AMMO, `class-type` 2): each one's `h_e5f51266` is the ammo-type that its guns'
  `ammo-type` names, so the box feeds every gun of that type (the reserve the weapon panel and
  the reload already use, play_ammo.rs `Reserve`):

  | Ammo (item type) | ammo-type | guns | `pickup-amount` | `stack-limit` |
  |---|---|---|---|---|
  | 11mm Ammo (h_12302d03) | 1 | Foley 356 Tact, L-Shot-50, L-Shot-75 | 10 | 50 |
  | High ROF Ammo (h_02735ad0) | 2 | Kman Auto, MK-ASLT, RVG50 Minigun | 50 | 600 |
  | Sonic Ammo (h_ef0c4c1e) | 3 | Jax-iP, Jax-iR, Jax-iC | 20 | 200 |
  | Particle Ammo (h_fc0d262e) | 4 | Saryl-23, Saryl-45, Saryl-99 | 25 | 200 |
  | Rail Ammo (h_e4286255) | 5 | Rail RVR, Rail CLVR | 10 | 400 |
  | Shotgun Ammo (h_e5d6b39e) | 6 | Bower 20 | 10 | 80 |
  | Cutter Ammo (h_fe018d0b) | 7 | Feral Cutter, Rusty Cutter, PU Cutter | 25 | 120 |
  | Bio Ammo (h_01c1965b) | 8 | MNR-7 Bio, A10 / A50 Bioreactive | 40 | 200 |
  | Rocket Ammo (h_11119186) | 10 | Sweeper V, Therm Sweeper | 8 | 40 |
  | Energy Ammo (h_15c866a4) | 12 | Ion Beamer | 20 | 200 |

  (Cutter and Rocket Ammo are the campaign's: the multiplayer maps place the other eight.) The
  ticket's column `h_0a811e94` ("a respawn time?") is the ammo-type written twice: the AMMO_BOX
  set's +0x44 returns it as such (0x147180). Respawns are the spawn-triggers' timers.
* **Taking one** is the walk-over (above): any living member of the squad in reach, the AI's too.
  The set's "can take" is the unconditional 0x280040, so nothing refuses a box because no squad
  gun uses the ammo or because the stack is full: `pickup-amount` goes into the squad's reserve of
  that ammo-type (FUN_0014ce90 with its multiplier flag, FUN_0014bfe0) up to the item's
  `stack-limit`, and
  * **something fits**: the box is removed for good, its pickup sound (h_e4097f62) plays at the
    character, PICKUP_OBJECT goes out from it, and the line "<n>x <name>" shows (n what went in,
    "Particle Ammo" alone for one): take01's box of 25 on a stack of 100 made the line "25x
    Particle Ammo" and the panel's reserve 38 / 100 -> 38 / 125 in the same moment (video 4.457
    s, Tex 1.26 m from the box's middle; take02: 1.21 m). A partial fit loses the rest: take02's
    stack of 195 took 5, "5x Particle Ammo", the box gone. Two boxes in a frame are one line,
    "20x Rail Ammo" (take03: two Rail boxes 0.5 m apart, taken in the same 50 ms, the two Rail
    guns 1.5-1.7 m away offering "Hold [X] to swap Rail RVR."). A stack nobody holds the gun of
    takes the type: take03's Rail stack was unassigned (count 0), the squad had no Rail gun, and
    "20x Rail Ammo" went in with no message;
  * **nothing fits** (the stack at its limit): the line "<name> (full)" in red, the box stays,
    and the walk-over finds it again every frame, so the line stays while the character stands
    there (take02, 19.8-25 s: "Particle Ammo (full)" under Tex, the stack 200 throughout). It
    goes to the HUD of the player who controls the character alone (FUN_0011c6a0 with AL clear):
    **an AI squadmate** standing on a full box shows nothing (take08, 14-22 s; the demo:
    `BF_TEST_TARGET=1`'s squadmate on it), and one who takes a box with room puts the line on the
    player's HUD, the reserve following (take08 3.26 s: Hawk on the box, "25x Particle Ammo" on
    Tex's half, 100 -> 125).
* **The respawn** of a multiplayer copy is its trigger's timer ([triggers.md](triggers.md),
  "Subject relays"): the Particle box 10 s after it was taken (take01 9.78 s of game time, take02
  9.8 s), the Rail boxes 20 s (take03 19.7 s), the demo's 10.00 and 20.00; a character in reach
  when it returns takes it at once (take04 for the Gas packs: 9 -> 10 by a returning pack; the
  demo, take02's stack: "(full)" from the frame it returns). The copy comes whole in a frame and
  the spawn flash's green rings rise over it for ~0.6 s (take01 15.5-16.1 s, take03 27.9-28.2
  s): the demo's flash is the same effect object (the medkits' too). A campaign level's box is
  a plain placement and never comes back (above).
* **Not done, or not the game's yet.** What `h_11884f2e` (20-50 on the ammo items) does: the
  AMMO_BOX set's +4 slot takes it with 3 (the stack loses it less 1), a caller nobody traced;
  not the walk-over, not the reload. The weapon panel's reserve text flashing 0.3 s at a pickup
  (`[[entry+0xf0]+0x98]+0x94 = 0.3`, FUN_0011c6a0): the widget isn't traced and take01's frames
  (4.5-4.9 s) show the "38 / 125" unchanged in colour and size; nothing is drawn. The AI's
  interest in boxes (the set's +0x54 / +0x58, 75 m, GOAL_GRAB_RESOURCE?) isn't traced: the squad
  takes boxes only by walking over them. The pickup effect: none (every ammo item's
  `pickup-effect` is empty). A dead enemy's drops (`inventory-drop`: the tutorial's terrorists
  carry "High ROF Ammo 48") are not made.
* **Test hooks:** `BF_PICKUP_LOG=1` prints each box taken (who, how many went in, the stack) and
  each one a full stack refuses (once). `BF_TEST_AMMO=<ammo-type or ammo name>:<n>[,...]` sets
  those reserves at the start, held to their stack limits (`4:195`, `rail:0`, `particle:200`).
  The test map's pickup row and its object menu (O) carry every ammo type.
* **Verified** (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_WEAPON="Saryl-99;Confed LZR-23"
  BF_TEST_GOTO=-51.70,82.10,-56.5,81.0 BF_TEST_WALK=1 BF_TEST_GO_AT=2.0 BF_PICKUP_LOG=1`, 330-420
  frames at 15 fps, take01's start and box): with `BF_TEST_AMMO=4:100`: "25x Particle Ammo" at
  4.00 s, the panel 38 / 100 -> 38 / 125, the box back at 14.07 s and taken again (the stack 150);
  with `BF_TEST_AMMO=4:195`: "5x Particle Ammo" (200) at 4.00 s, and at 14.07 s the box returns
  with the green rings and "Particle Ammo (full)" in red stays up under Tex while he stands
  there (frames 222-267), as take02; take03's start
  (`BF_TEST_AMMO=rail:0 BF_TEST_GOTO=19.53,70.07,13.0,69.5`, 480 frames): "10x Rail Ammo" at
  4.13 s and 4.20 s, "20x Rail Ammo" on the one line, the stack 20, the two boxes back 20.00 s
  later (24.13 s and 24.20 s); take08 (`BF_TEST_TARGET=1 BF_TEST_GOTO=-55.98,89.09,-55.98,96.0`,
  the squadmate standing on the box): Brutus takes it at once, "25x Particle Ammo" on Tex's HUD
  and 38 / 125, and with `BF_TEST_AMMO=4:200` nothing shows under the standing squadmate for the
  whole run. The tutorial's two medkits (`BF_MAP=tutorial BF_TEST_GOTO=51,14,51.0,18.0`): taken
  at 1.51 and 1.55 s, and not again in the minute after; its DNA canister at 1.74 s (1.80 s with
  the old 1.3 m).

## Grenade packs, the "Bio Ammo" of 3 (#19)

`play_item_pickups.rs` again (the same walk-over, "Ammo boxes" above), from the same leads and
`todo/18-ammo-pickups` takes 04-07 (sdm_e34, xemu).

* **What they are.** Four inventory types the data names "Bio Ammo" (stringtable-name
  h_fca63a02: the ammo box's string, copied), `function-type` 0 (the base set, not the ammo
  box's 14), `group-type` 1, `pickup-amount` 3, `stack-limit` 999, each with a
  `pickup-archetype` that is a grenade: the pack gives *that*, so what goes into the store is
  the archetype's, 3 of it, and the pack's own 999 and key (1, 20-24) are never used:

  | Pack | grenade it gives (its key) | the line | on the multiplayer maps |
  |---|---|---|---|
  | h_fd437c4d | Frag h_060f95f7 (2, 0) | "Frag Grenades" | 5 |
  | h_11291ea2 | Gas h_e8c67904 (2, 1) | "Gas Grenades" | 33 |
  | h_eb44e4f4 | Sonic h_ec0bf28f (2, 4) | "Sonic Grenades" | 40 |
  | h_1913256a | Energy h_0cc3c2c8 (2, 5) | "Energy Grenades" | 32 |

  (And the Light's, h_fa558016 -> h_fd1a966d, "Flares", on m09_b. The campaign places the packs
  too: the Frag's in 30 levels, the tutorial's three at (-62.5, 15.6, 20.8), (-63.3, 15.6, 20.0)
  and (51.1, 16.1, 20.0).) The grenades are the demo's grenade kits (`GrenadeKits`, the squad's
  (2, index) stacks `Player::grenades`), each held to its `stack-limit`, **10**.
* **Taking one** is the walk-over as the boxes' (any living member in reach, the AI's too, the
  same reach): 3 of the grenade go into its stack up to 10 (FUN_0014ce90 with the pack's
  `pickup-amount` as the multiplier, the archetype's class-type and stack-limit; FUN_0014bfe0),
  the pack is removed, its sound plays, PICKUP_OBJECT goes out and the line is the grenade's
  display name (h_1dd3a49d of the type FUN_00146340 gives, not the pack's "Bio Ammo"):
  * take04: Tex with no grenades (the Gas stack at 0, no type) walks through three Gas packs
    lying within 1 m of each other: "3x Gas Grenades", "6x", "9x" on the one line, the box (it
    was empty) selecting Gas and showing "NEW 3", then 6 and "NEW 9" ([play.md](play.md), "The
    item box": the label is the stack's first showing), the packs gone. The pack that returned
    10 s later, with Tex standing there and the stack at 9, was taken at once: 10.
  * take05: the stack at 10: red "Gas Grenades (full)", the three packs stay and the line stays
    while he stands there.
  * take06: the stack at 8: the first pack gives "2x Gas Grenades" (10) and is gone; the other
    two stay. The line is blue for 4 s, and **then the red "Gas Grenades (full)" appears under
    it** (8.65 s: the walk-over keeps finding the full stack, and a count-0 line can only start
    once the "2x" line has run out: FUN_000ad790's rule, `PickupFeed::add`).
  * take07: two Energy packs 0.7 m apart, the Energy stack empty and typeless: "3x", then "6x
    Energy Grenades" (the (2, 5) stack). Sonic packs weren't filmed (all three clusters sit
    against rock): the same code.
* **Not done.** The multiplayer maps' 75 Roller and Sentry spawns, `<h_1d1d4f05>` placements of
  object-type 24 types (h_15f9c51a "Roller": the carried Roller (3, 4); h_ecfe0d8c "Sentry": the
  carried Sentry (3, 3); stack-limit 10 each, `cost` 350 and 500), all spawn-trigger templates:
  the game takes them by the same walk-over (+1 each), but `spawn_copy` copies inventory-objects
  and weapon-objects only, so none shows (#278, above): a ticket of its own. The pack of a
  grenade type the map's kits lack (the Light's off m09_b) isn't taken.
* **Verified** (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_INV=Frag:0 BF_TEST_GOTO=-77.52,15.19,
  -78.3,11.4 BF_TEST_WALK=1 BF_TEST_GO_AT=2.0 BF_PICKUP_LOG=1 BF_ITEM_LOG=1`, 330 frames at 15
  fps; `BF_TEST_INV=Frag:0` takes the demo's start Frags away, as the takes' squad had none):
  taken at 3.47, 3.60 and 4.07 s, "3x" -> "6x" -> "9x Gas Grenades" on one line, the box
  selecting Gas at 3.47 s with NEW (the log: "none -> grenade 1", "NEW on grenade 1"), the same
  as take04's 3.7-5.5 s frames; the first pack back at 13.47 s and taken: 10, the other two full.
  With `BF_TEST_INV=Frag:0,Gas:10`: "Gas Grenades (full)" red, the packs staying (take05); with
  `Gas:8`: "2x Gas Grenades" at 3.47 s, the red "(full)" from 8.0 s, the box 8 -> 10, the two
  packs staying (take06's 4.15 and 8.65 s); take07's start (`BF_TEST_GOTO=-66.8,37.2,-71.5,38.2`):
  "3x", "6x" Energy Grenades.

## OrgSen (#21)

`play_item_pickups.rs` (the pickup), `play_vision.rs` (the use), `play_vengar.rs` and
`src/vengar.wgsl` (the view), `play_reticle.rs` (the lock), from `todo/18-ammo-pickups/leads.md` §4
and checked against that folder's take09 (taking it) and take10 (using it), sdm_e13, xemu.

* **What it is.** "OrgSen" (h_f637c4c8; its description h_f9d7fa96: "Easily detects concealed
  organic signatures even through most obstacles.") is two `<inventory>` types of sdm_e13's (and
  16 other levels') objecttypes: the **carried** one, h_11433446 (function-type 12
  IFSET_VISION_ASSIST, group-type 6 IG_ENHANCEMENT, stack index 0, stack-limit 1, `h_1ee2f4ed` 1
  IOU_ATTACH_TO_FACE, `cost` 5000, icon h_1b73c691, mesh h_0a9b2b06), which the campaign's squad
  placements carry (#281's "OrgSen 1 NEW" on e34 and e40), and the **placed** one, h_05a87396
  (function-type 0, group 6, stack index 23, pickup-amount 1, pickup-archetype the carried one),
  which only sdm_e13 places, as two spawn-trigger templates: (4.3, 29.2, 27.4) and (-84.8, 27.9,
  -176.8); the live copy of the first lies at (2.95, 27.38, 27.74), 10 s timer. The ticket's
  guess that it's an objective item is wrong: nothing but the vision mode reads it, and no game
  mode is checked (the same on the campaign's zones as in the multiplayer map).
* **Taking it** is the walk-over of the ammo boxes (above): the base set's "can take" says yes,
  1 of the carried type goes into the squad's (6, 0) stack, held to 1, the pickup is removed, its
  sound plays, PICKUP_OBJECT goes out, the line is "OrgSen". The box (empty before) selects it and
  shows "NEW OrgSen" for 2 s, blinking (take09 4.6-6.6 s). With one carried: the red "OrgSen
  (full)" and the pickup stays (take09, take10's first seconds: Tex starts on the returned
  copy). The copy comes back 9.9 s after (take09; the demo's relay 10.00 s) and a character
  standing there is told "(full)" again.
* **Using it** (the use button, LT; the item box selects it first): FUN_00121170's case 1.
  * With nothing on the face: the stance's `bring2face` overlay (motion type 10, OMT_BRING_TO_FACE;
    `Sc_w1_` / `Sc_w2_` by the weapon in hand), Tex's 1.80 s, Hawk's 3.00, Flint's 3.43, Brutus's
    3.00, the upper body, the character free to move. Its first event (1a6b4920, 0.57 s of Tex's,
    0.43 Flint's, 0.67 Hawk's) is the OrgSen in the hand holder, as a copy (the stack keeps its
    1: FUN_0014abc0 takes nothing out for use types 1 and 7); at its **attach_item** (0x0775fceb,
    1.33 s in Tex's, Hawk's and Flint's first stance, 1.67 in Flint's second) it moves to the face holder and
    its put-on hook sets vision mode 1 (FUN_001479f0: char+0x644 = 1 and the camera's +0xd8 = 1).
    take10: LT at 3.0 s of video, the hand holder from 3.8 s and the face holder from 4.66 s,
    the picture thermal at 4.7 s: 0.67 and 1.42 s of game time after the press, the clip's 0.57
    and 1.33 s (the demo: the clip starts a frame after the press and the vision is on 1.34 s
    later: `BF_PICKUP_LOG`'s "bring2face clip starts" at 3.13 s, "vision mode 0 -> 1" at 4.47
    s, for LT at 3.0). A Hawk who is stealthed pays 5 stamina at the start (the skill's +0x104).
    The box stays pale blue and "OrgSen" through the clip (the colour is the skill's "can use",
    not "a use is pending": take10 3.0-4.6 s) and after; the count of 1 isn't drawn.
  * With it on: LT takes it off **at once**, no clip (FUN_00121010: the item's state 5, then the set's
    "taken off" hook 0x147a20 -> FUN_00126df0(character, 0)). Setting a state plays that state's
    event record of the item's type (FUN_0014a610): the OrgSen's state-5 event names the sound
    h_feaf631d, played at the take-off (take10's audio: a short broadband transient 40 ms after
    the press at 17.0 s, +25 dB above 1 kHz; the demo queues it at the press). The put-on sets the
    item's place state 2 (FUN_0014a880), which has no event record: no sound there, as the take
    has none at 4.66 s. take10: LT at 17.0 s, mode 0 and the normal picture the frame after
    (17.10 -> 17.20 s).
  * The gate is the item use's (FUN_001210c0: the character free, nothing in the second hand, the
    skill's word), as the Medkit's. **Brutus's skill refuses the use type**: "Cannot use", the box
    red (#196 take26). The OrgSen worn belongs to the character: another squad member's view has
    its own mode; a dead character's is dropped (a guess: the death camera is another view).
* **The view, vision mode 1** ([skills.md](skills.md), "The Vengar view": the same renderer
  branch, FUN_000e81e0, with its own world pass FUN_000e6a70 where mode 2 has the blur, the same
  vision lights and keyed terrain, and a last full-screen quad FUN_000a45a0). take10's picture,
  which fits what the code says of it: the world's luminance mapped onto a palette of dark
  blue-purple to magenta (the lava, the brightest, (190, 66, 214) at the scan lines' peaks); the
  living **and the dead** characters over it in a yellow figure (the heat figures' material,
  the same radial ramp: yellow-white (240, 227, 62) facing the view to orange-red (182, 76, 41) at
  the rim) with a halo, the gun in the hand a dark dithered purple and red; and over everything
  scan lines: a cycle every 4 pixels of the game's 480 lines, the picture times 0.27 at the
  darkest to 1.0, the HUD not under them. The switch is at once both ways (no colour fade: the
  red fade is Vengar's alone). The palette, the figures' ramp and the scan lines' profile are
  **fits** to take10 (the materials are made of their classes' defaults and are in no archive,
  FUN_000a42d0; the numbers are at `vengar.wgsl`'s `thermal` and `scan_lines` and
  `play_vengar.rs`'s `THERMAL`, `CARRIED` and `SCAN`, with the measurements). **The halo is not
  one of them**: mode 1 draws Vengar's (`play_vengar.rs`'s `GLOW`, 0.07 a tap, a fit to #122
  take09's Brutus), and its size and strength were never measured on take10. Against take10's
  terrain (the cycle averaged) the demo's mean is (37, 15, 70) for (36, 14, 73), its 10th / 50th /
  90th percentiles blue 50 / 66 / 94 for 43 / 72 / 104. `BF_THERMAL_RAW=1` shows the vision-lit
  picture the palette is fitted on. The figures are drawn against the world's depth, so
  **an enemy behind a ridge isn't shown** (take10: one 11 m away and 6 m higher behind the rock
  ridge, and two bodies burning in the lava, weren't: the look doesn't show through walls; the
  lock does).
* **The lock through walls.** With the OrgSen on and a gun in hand whose bullet passes through
  what it hits (the Rail RVR, the Rail CLVR and the Saryl-45; the others' h_11caf57b is false):
  FUN_000bf5a0 sets the target selector's "no sight ray" flag, and FUN_000bfeb0 (0xbfbf3) lets a
  character within 75 m of the controlled one (3D, the positions) through without it, so the
  crosshair locks on one behind a wall or another body; and the aim point (0xc5720) is the
  lock's, with no world ray. (`play_reticle.rs` `sees_through`; the test `the_orgsen_sees_
  through_within_75_m`.) Verified on sdm_e13's test session (`-- --test BF_MAP=sdm_e13 BF_TEST_WEAPON=
  "Saryl-45;Confed LZR-23" BF_TEST_INV=OrgSen:1 BF_TEST_ITEM=OrgSen BF_TEST_ITEM_USE=0.5
  BF_TEST_NPC=<a ring of frozen enemies> BF_TEST_SWEEP="0;1.0,11.0,36" BF_RETICLE_LOG=2`): an
  enemy standing 6 m behind another on the crosshair's line, "hidden" with the OrgSen off, is
  not with it on.
* **Blocked or not done.** A Rail's *shot* through a wall at a locked enemy: the lock sees it, but
  the demo's pass-through rays stop at an object's far side (play_rays.rs: n1 1) and no take has
  the shot (take10's notes: "Not recorded ... the Rail-lock-through-wall test with a gun in
  hand": a Rail gun wasn't staged), so whether it reaches is unmeasured. Whether the dead, or
  the characters of the other team, show differently isn't filmed (take10 had one living enemy
  in the open and the dead in lava); the demo draws every character the same. The state 4
  event's sound (h_f37f35df) isn't reached by these flows (which change of an item's state sets
  4 isn't traced), so it isn't played. The OrgSen's model in the hand and on the face (the
  holders' hardpoint h_08d7f793) isn't drawn. Flint's scope overlay, which isn't drawn while a
  mode is on (FUN_000b3970), isn't hidden. The mode isn't dropped by anything but death (and a
  respawn). The Hat (use type 7, IFSET_AIR_FILTER) is still unused.
* **Test hooks.** `BF_TEST_INV=OrgSen:1` carries one (the carried type), `BF_TEST_ITEM=OrgSen`
  selects it, `BF_TEST_ITEM_USE=<s>[,<s>...]` presses LT at those times (put on, then off),
  `BF_PICKUP_LOG=1` prints each use and the mode's changes, `BF_SKILL_LOG=1` the view's
  (`vision mode 0 -> 1`), `BF_THERMAL_RAW=1` the view without its palette.
* **Verified** (`BF_MAP=sdm_e13 BF_CHARACTER=3 BF_TEST_INV=Frag:0 BF_TEST_GOTO=7.34,26.75,3.3,27.6
  BF_TEST_WALK=1 BF_TEST_GO_AT=2.0 BF_PICKUP_LOG=1 BF_ITEM_LOG=1`, 330 frames at 15 fps): taken
  at 4.07 s, "OrgSen" under Tex, the box selecting it with NEW blinking for 2 s, the copy back at
  14.07 s and "OrgSen (full)" red from 14.13 with the green rings (take09). `BF_MAP=sdm_e13
  BF_CHARACTER=3 BF_TEST_INV=OrgSen:1 BF_TEST_ITEM=OrgSen BF_TEST_ITEM_USE=3.0,17.0
  BF_TEST_GOTO=3.51,27.48,3.51,27.48 BF_PICKUP_LOG=1`: the clip at 3.13 s, mode 1 at 4.47 s, off
  at 17.07 s; frames 40-100 the world dark purple, the lava magenta, the scan lines and Tex a
  yellow figure with a halo, as take10's 4.8-17.0 s (`BF_TEST_SWEEP=102 BF_CAMERA_PITCH=-0.05`
  turns the camera to take10's view). Flint's and Hawk's clips attach at 1.34 s; Brutus is
  told "Cannot use".

## Pickups from spawn-triggers (#278)

`play_pickup_spawns.rs` (the copies), `play_triggers.rs` (`spawn_copy`, the subject relay), from
default.xbe's code (the addresses below) and checked against #35 take20 (sdm_e34's STAMINA POWER)
and #129 take38 (mp1's walkway).

* **What they are.** Every pickup the multiplayer maps place is a spawn-trigger's template
  (sdm_e34 54 of 54, mp1 29 of 29 ... sdm_e40 66), and so are the campaign's supply drops and
  caches (e34's Therm Sweeper, medkits and ammo after a radio line; m02_a's 15, m04_c's 20 ...).
  A template is out of the world (FUN_000dfe90): it isn't drawn, taken or counted where it's
  placed, and its idle effect doesn't start there. Its model is kept hidden
  (`level_scene::SpawnTemplate`) to draw the copies with. The level's other pickups (the
  tutorial's all) are as before.
* **The copies.** Each spawn of the trigger (on the multiplayer maps: its FIRST_UPDATE, then each
  TIMER_PULSE, with OPERATE and SPAWN_RESET; [triggers.md](triggers.md), "Spawn-triggers") puts a
  copy of the template at the trigger's transform (FUN_00209620; an item's place call is the game
  object's 0x159b70, with no ray to the ground), named by the trigger (a re-armable one's get
  made-up names, as the game gives them none):
  * a type without a collision body (`collision-type` CT_NONE, the enum 0x3bd79c: the four
    power-ups' orb h_e9e61b85) stays at the transform with its idle effect: sdm_e34's STAMINA
    POWER at (-84.07, 29.78, 60.58), where take20's live object lies, 2 m from its placement
    (-82.11, 32.06, 60.06). Walking over one takes it ("Power-ups" below, #15);
  * another item (CT_CONVEX) lies on the ground under the transform, upright and tilted to the
    ground, loose, as the level's own pickups do ("Pickups are loose" above); medkits and fruit
    can be taken (no level has a DNA canister as a template). The multiplayer maps' triggers stand
    ~0.5 m over the ground, so the copy is the body the game's physics drops there, at rest (its
    fall isn't drawn);
  * a weapon lies on its side on the ground under the transform, turned the trigger's way about
    the vertical (take38: the guns flat on the walkway under their green spawn glows; where the
    game's physics rests it isn't read), a lying gun with its type's full clip ("Weapon pickups"
    above: the prompt, the swap, the walk-over).
* **Taking one** removes it for good (FUN_0011c170 removes an item it takes; the grab the gun) and
  sends PICKUP_OBJECT from it {iD 0, the taker, data -1}. A gun copy walked over keeps lying with
  what's left (its 10 s vanish timer outside a mission, above).
* **The respawn** is the data's ([triggers.md](triggers.md), "Subject relays"): the trigger's
  OPERATED goes through a subject relay that binds the copy to a timer, the PICKUP_OBJECT starts
  the timer, and its pulse makes the next copy and, beside most, the spawn flash (an effect
  object of type h_117c1805). sdm_e34: the STAMINA POWER 30 s (h_0021e47f; take20: back ~31 s
  after it was taken, fading in over ~0.3 s), its medkits 10 or 45 s, its guns 20 or 45 s, its
  ammo and grenades 10 or 20 s, BRUTE POWER 180 s, FORCE POWER 165 s, HEALTH POWER 30 s.
* **Not done:** the copy's fall and its fade-in (take20's ~0.3 s; the spawn-trigger's h_eeae96e7
  is 2 s, the demo draws no fade; take01 of #18 saw the Particle box appear whole in one frame,
  then the green rings of the spawn flash for ~0.6 s); where
  the game's physics rests a gun copy (lying on its side is take38's look); the multiplayer maps'
  75 grenade pickups, item spawns `<h_1d1d4f05>` (types h_ecfe0d8c and h_15f9c51a, object-type
  24): their spawn-triggers fire and their relays bind, but `spawn_copy` copies inventory-objects
  and weapon-objects only, so none shows (none did before either: #230 hid them as templates).
* **Test hooks:** `BF_PICKUP_LOG=1` (or `BF_TRIGGER_LOG`, `BF_WEAPON_LOG`) prints each copy made
  (its trigger, name, template, type, place and kind), the item copies listed and each taken.
* **Verified** ([triggers.md](triggers.md) "Verified" has the commands): sdm_e34's 54 copies at
  the start, the STAMINA POWER's ahead left of Tex from take20's spot; three medkits taken and
  back 10.00 s later with the flash; a Saryl-45 swapped and back 20 s after the grab, the dropped
  LZR-23 gone 20 s after the release; mp1's walkway against take38's player 2 view (the
  L-Shot-75 on its side at Flint's feet, the medkits 14 m on); the tutorial and e34's own pickups
  as before.

## Power-ups (#15)

`play_powerups.rs` (with `play_hud.rs`'s bar flash and `src/shell.wgsl`), from default.xbe's code
(the addresses below, each in [code-map.md](code-map.md)) and checked against #35 take20
(sdm_e34's STAMINA POWER) and #85 take28 / take29 (sdm_e34: a barrel at 1 m with and without FORCE
POWER).

* **What they are.** BRUTE POWER, FORCE POWER, HEALTH POWER and STAMINA POWER, the glowing orbs of
  the multiplayer maps (none in the campaign): each a spawn-trigger's copy at the trigger's
  transform with its idle effect ("Pickups from spawn-triggers" above). Their item sets, by
  function-type: HEALTH and STAMINA POWER 19 IFSET_POWERUP_MEDKIT (h_0a811e94 health 50 / 0,
  h_11884f2e stamina 0 / 50), BRUTE 20 IFSET_POWERUP_BRUTAL_DAMAGE (2 and 45), FORCE 21
  IFSET_POWERUP_FORCE_SHIELD (50 and 45). sdm_e34 has one of each: BRUTE at (-0.70, 29.98,
  83.69), HEALTH (-12.30, 29.58, 77.83), STAMINA (-84.07, 29.78, 60.58), FORCE (-49.16, 29.78,
  38.85).
* **Taking one** is a walk-over (FUN_0011c540 -> FUN_0011c170), by any living member of the squad,
  the AI's too: the orb at most 1.8 m below and 1.0 m above the character's position (the hips) and
  under 1.34 m across (dx^2 + dz^2 < 1.8), as for a lying gun.
  * HEALTH and STAMINA POWER only when they can do something (their set's "can take", 0x148d40):
    the character hurt, for HEALTH; its stamina below the most, for STAMINA. Refused, the HUD of
    the player whose character it is flashes the bar it would fill (0x146900): the fill turns amber
    (the bar widget's flash colour +0x59, bytes e6 aa 3c) four times, 0.15 s each, 0.3 s apart,
    the first 0.15 s after the walk-over, and no new flash can start for 5 s more (its timer +0x48
    runs 6.2 s). take20, walked over at full stamina (2.0-4.2 s): the stamina fill amber at
    3.50-3.60, 3.80-3.90, 4.10-4.25 and 4.45-4.55 s. The amber is each texel's luminance x the
    colour x 1.64, fitted to take20's pixels (the game's colour operation isn't traced; the
    low-stamina blink's red fits the same). An AI member refused shows nothing.
  * BRUTE and FORCE always (their sets' 0x280040).
  * Taken: the item's pickup sound at the character (STAMINA h_e2824511, HEALTH h_e58a6855, BRUTE
    h_1959b73b, FORCE h_f5581a7c; FUN_00146190), PICKUP_OBJECT from the copy (its trigger's relay
    times the next one: sdm_e34's STAMINA and HEALTH POWER 30 s, BRUTE 180 s, FORCE 165 s), the
    pickup line with its name ("STAMINA POWER", 4 s: take20 9.48-13.6 s), and it's taken out of
    the world (FUN_000db3e0). Its idle effect's particles live out their lives where it was:
    take20's blue glow at Tex's feet (9.4-9.9 s).
* **HEALTH and STAMINA POWER** (0x148de0): health + 50 up to the most, or stamina + 50 up to the
  most (take20: 21.60 -> 71.68). No chatter: the "healed" line is the medkit's set's (0x146f40).
* **BRUTE and FORCE POWER** (FUN_00148b30): a modifier on the character for 45 s (h_11884f2e),
  counted down each frame, alive or dead (FUN_0011a840). Taking one of a kind that's on only puts
  its time back to 45 s (FUN_0011c170 at 0x11c458).
  * BRUTE: every damage roll the character makes x2 (h_0a811e94; FUN_0022c940's hooks, 0x15f670):
    its shots, bolts, rays and missiles, their blasts and its shots on scenery. **x2, not "quad
    damage".**
  * FORCE: every hit the character takes x0.5 (50 x 0.01), but a BLADED one (DTYPE 2) whole
    (0x125600's hooks, 0x15f7c0): shots, blasts, damage areas, gas, liquids, falls. take28: a
    radiation barrel at 1 m took 19.0 of Tex's 70 where take29's took 39.4 (x0.48).
  * Both at once multiply. `BF_COMBAT_LOG` names them: "x the shooter's power-up 2.00", "x the
    victim's power-up 0.50" (a damage area's line: "x0.50 its power-up").
  * No HUD shows the time: nothing but those hooks reads the list.
  * The "on" also sends a commentary cue (0x0f5428da BRUTE, 0xf9994826 FORCE) that every
    commentary file lists with no sound and no text: nothing plays.
* **The shell** (`ShellMaterial`, src/shell.wgsl). While a modifier lasts, its character's
  material controllers of its kind (8 for BRUTE, 0x10 for FORCE) draw the body again with the
  item's state-0 event material (`<events><event state="0" h_05db8834=..>`: BRUTE's h_1959b73b,
  FORCE's h_f5581a7c), a wrapper of a shell material (class h_0112da6f):
  * the body pushed out 0.03 m along its normals (`offset`); its guns aren't drawn so (take28's
    gun stays grey);
  * added to the picture (ONE / ONE): the texture x the colour, BRUTE's red arcs h_fab1de97 x 0.5,
    FORCE's blue arcs h_0f91fbc8 x 0.8;
  * the texture looked up by the surface's normal in view space and scrolled by the material's
    clock, BRUTE's down 0.6 a second (`translate-v` -0.6), FORCE's up 0.8;
  * the wrapper's glare key, BRUTE 120 and FORCE 150: what's brighter than that under the shell
    blooms ([levels.md](levels.md), "The glare").
  take28: "Tex glows blue-white" from start to end, his gun grey; so does the demo's. It shows from
  the take's frame and stays 1 s past the modifier's end (the controllers' fade-out, which they're
  drawn through). The game gives those controllers to the multiplayer squad types (mp_brutus,
  mp_hawk, mp_tex, mp_flint) and not to the campaign's brutus and hawk; the demo's squad plays the
  campaign types on every map, so it draws the shell on anyone. Each body wears an invisible
  shell for 1 s as its model first shows, so bevy has built the shell's pipeline before a take
  (without it the first shell came 1.1 s after its take).
* **Not done, or not the game's yet:**
  * the orbs' look: the icons' flares are GeneralApp_FacingType 2 (square to the particle's
    motion) on sphere emitters throwing them out from the middle (0.025 m/s), faded near edge-on
    (AngleFadeOut): a glowing ball, as take20 shows at 2.0 s. The demo lays them flat in the
    emitter's frame (ale_fx.rs's perp rule): a flat disc, thin from a low camera. That's the
    facing types' work (#286), not the power-ups';
  * the reach against take20: the take came between 1.11 and 0.98 m across (the watch log's
    positions at 9.29 and 9.34 s) where FUN_0011c170's 1.34 m would have taken it about 0.08 s
    sooner; the refusal's flash (3.50 s, less xemu's ~0.13 s picture lag) fits 1.34 m. The code's
    reach is used; what the take measures it from isn't known;
  * the shell isn't fogged (the game's program fogs it by depth);
  * BRUTE on the grenades: their damage is the demo's own rule (#112), not FUN_0022c940's roll;
  * BRUTE's and FORCE's take and end aren't filmed: they're the code's (a shot list: BRUTE taken
    in the open with the camera on the taker, a few shots into a target, then the 45 s out to the
    shell's end; FORCE the same, with a hit taken).
  * the shell during Brutus's Vengar view: the shell is made from the body's `StandardMaterial`
    parts, which the view swaps to its heat material, so no shell shows while the view is on
    (review of #15; multiplayer power-ups and the campaign skill rarely meet);
  * a pickup line on the player's HUD when an AI squad member takes one: FUN_0011c6a0's line goes
    to the player controlling the character (to every player of the squad with AL set); whether
    an AI's take shows one isn't read (the tester of #15).
* **Test hooks:** `BF_TEST_POWERUP=<brute|force|health|stamina|h_<type>>[@<s>][;...]` gives the
  controlled character that power-up at that time (0.5 s by default) as a walk-over would, refused
  (the bar flashing) when it can't take it: the map's (or the test map's) first item of that set,
  HEALTH and STAMINA the orb that gives that. `BF_PICKUP_LOG=1` prints each power-up taken and what
  it did, each modifier's end and the shells made; `BF_COMBAT_LOG=1` the factors on each hit.
* **Verified:**
  * take20 at full stamina (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_GOTO=-81.78,63.86,-84.6,59.8
    BF_TEST_WALK=1 BF_TEST_GO_AT=0.5 BF_CAMERA_PITCH=-0.5`, 60 frames): Tex walks over it, it
    stays; the stamina fill amber at 2.33-2.40, 2.60-2.73, 2.93-3.00 and 3.20-3.33 s, as take20's
    four, the same amber against the same teal;
  * the same with `BF_TEST_STAMINA=20@0.2 BF_PICKUP_LOG=1 BF_TRIGGER_LOG=1`: taken at 2.27 s
    (stamina 72.48, +50), the orb gone, "STAMINA POWER" under Tex, the bar rising pale,
    PICKUP_OBJECT to h_0021e47f (30 s); in real time (no capture) the next copy at 32.23 s;
  * FORCE at take28's spot (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6
    BF_TEST_HIT=2.0 BF_TEST_POWERUP=force@1.5 BF_COMBAT_LOG=1`): Tex took 20.0 of the barrel (40.0
    without, scenery.md's take29 recreation), the shell from 1.53 s, the take's frame;
  * BRUTE on the test map (`-- --test BF_CHARACTER=3 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=0,8,0,7.3
    BF_TEST_NPC=0.3,flint,enemy,dummy,0.15,3 BF_TEST_FIRE=hip BF_TEST_POWERUP=brute@2.0
    BF_COMBAT_LOG=1`): 4.29 and 6.32 before the take, then "x the shooter's power-up 2.00" on
    every hit, 9.61-12.48;
  * BRUTE and FORCE together in the open (`BF_TEST_POWERUP="brute@0.5;force@2.0"`): the red shell,
    then red and blue adding to white; "BRUTE POWER" over "FORCE POWER".
