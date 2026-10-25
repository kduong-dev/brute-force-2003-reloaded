# Health pickups

* **Health pickups** (`play_pickups.rs`). Placed `inventory-object`s whose item type (objecttypes
  `<inventory>`, `Game::items`) has function-type 5 or 19. Function-type is the game's IFSET_
  enum, named in default.xbe's table at 0x3be3cc: 5 IFSET_GENERIC_HEALING, 14 IFSET_AMMO_BOX,
  16 IFSET_MINIGUN, 19 IFSET_POWERUP_MEDKIT, 20-23 the power-ups.
  * Medkit (h_f5123ace): taken by walking over it (within 1 m) into the squad's shared
    inventory, up to its stack-limit of 25, with its pickup-sound. When full, "<name> cannot
    pick up Medkit." shows and it stays.
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
    full health. HEALTH / STAMINA POWER are also function 19, but they're power-ups with idle
    effects and aren't taken yet.
  * A taken pickup comes back after 30 s (a guess; the respawn time isn't found). Only the
    player picks up.
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
  member whose feet come within 1.3 m of it across (and 1.5 m up or down) takes it: a fit to
  take12, where the line came between Tex's feet at 1.41 m (295.30 s, no line at 295.25) and
  1.25 m (295.44 s, the line fading in at 295.5); the game's test isn't traced. Its pickup sound
  plays, the line shows its display name, and it's gone for good (an unlock is once).
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
    MK-ASLT and m02_c's iKhan-GPL are its copies' (#278), drawn where they're placed as before;
  * the gun a swap drops, and a dead character's guns (both: release_weapon's dead path,
    FUN_00117660 at 0x118a4a), with the rounds they had; they fall and tumble as the loose
    pickups do ("Pickups are loose" above).
  * Taken, a gun is gone for good: nothing puts a placed campaign weapon back (leads §4.6), so
    the medkits' guessed 30 s respawn isn't theirs.
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
    6.57 / 6.88 s, take11 300.73 s).
  * The prompt for the next gun is back from the grab, while the clip plays on.
* **Walking over a lying gun** (FUN_0011c540 -> FUN_0011c170), any living squad member, the AI's
  too: within 1.8 m below and 1.0 m above its position and under 1.34 m across, a gun with rounds
  whose ammo type a squad member's gun uses gives them to the reserve, keeping what doesn't fit,
  with the ammo item's pickup sound, PICKUP_OBJECT from it and the line "2x Sonic Ammo"; with the
  stack full nothing goes and the line reads "Sonic Ammo (full)" in red (a0 20 20), every frame
  it's walked over (the DNA video's "Bio Ammo (full)" and "2x Sonic Ammo"). In #61 and #52's
  takes nobody alive used the dropped guns' ammo: no line, as in the demo.
* **The AI never swaps guns**: no goal of the game's picks one up (leads §9).
* **Not done, or not the game's yet:**
  * where a dropped gun lands: it falls from the hand with a small spin (the demo's; the game's
    drop, FUN_0014a1d0's physics, isn't read: shot list C);
  * the vanish timers a dropped gun gets outside the game mode 0x17506391 (20 s, 10 s after a
    walk-over): the campaign's stay in every take; the mode isn't identified (shot list C2);
  * the use's walk to the gun (FUN_00120290's phase 1, when the gun is beyond 2-3 m of a slot in
    hand or 1 m otherwise): the demo uses it from where the character stands;
  * take11's view turns about 20 degrees during the use (300.6-301.2 s, no stick input): not
    traced, not done;
  * the faint grey shape over the weapon panel between the release and the grab;
  * the game's slot left empty by a use cut short after the release: the demo takes the gun at
    once instead; a character with fewer than two guns doesn't pick into the empty slot;
  * BEFORE_USE_OBJECT (18) at the use's phase 2 isn't sent (nothing the levels have listens);
  * the shot list's open questions (leads §0): a part-used lying clip (the code reloads it at
    once at the grab), the hold time's edge, a swap while scoped.
* **Test hooks:** `BF_TEST_GUN=<weapon>[,<ahead m>[,<right m>[,<rounds>]]][;...]` lays those
  guns round the controlled character's start, 3.5 m ahead by default (the takes' carrier was put
  there and killed), full unless given rounds; `BF_TEST_SWAP=<s>[,<s>...]` holds X 0.5 s from
  each time (`BF_TEST_USE=<s>` holds it 1 s); `BF_WEAPON_LOG=1` prints the lying guns, the
  target as it changes (its class, the slots' classes and the mode), each use, its clip, the
  release and the grab with the rounds and the reserve, the walk-overs and the dead's drops.
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
