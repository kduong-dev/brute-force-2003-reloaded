*Part of [pickups/](README.md), the former page `pickups.md`.*

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
    ([triggers.md](../triggers/object-events.md), #220; none listens to a medkit in the levels played so far).
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
    [play.md](../play/item-box-and-item-list.md), **The item box and the item list** (#196).
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
    1a6b4920 reach, 0a6e8f79 use, 19f8311b let go) and works in three steps (#361, FUN_00117660;
    the demo had the events' roles wrong and healed at 19f8311b, 0.5-0.7 s late):
    * **At the press** the stack's count drops (FUN_00121170 -> FUN_0014abc0; #196 take13's
      Medkit 2 -> 1 in the first log row after the press): `use_medkit`.
    * **At 1a6b4920** the medkit (the carried type's model, h_192d5337's archetype, its cross
      turned red: its texture's blue texels, and its cross glow red) is in the throwing hand.
    * **At 0a6e8f79** (0.13-0.27 s in: Brutus 0.13, Tex 0.17 in the w2 stance and 0.27 in w1,
      Hawk 0.17, Flint 0.23) it heals (health + 80, one add: FUN_00117660 -> 0x14b980 ->
      0x146f40, no ramp; take13 hp 60 -> 115 in one 50 ms row, the bar full 0.35 s after the
      press) and plays the medkit sound and the "healed" line: `medkit_used`.
    * **At 19f8311b** (0.77-0.90 s) the used medkit is let go (FUN_001215e0): it falls from the
      hand, bounces and stays on the ground (the last 12 do). It heals nothing.
    Checked (#361, `BF_TEST_HEALTH=60 BF_TEST_MEDKIT=1.5 BF_TEST_INV=Medkit:3 BF_TEST_ITEM=Medkit
    BF_PICKUP_LOG=1 BF_CAPTURE_FPS=60`, e34's range spot): the press at 1.52 logs "2 left", the
    heal at 1.65 (Brutus), 1.68 (Hawk, Tex), 1.75 (Flint, the L-Shot-75's w1 stance), and the
    medkit let go at 2.33 (Tex, 0.8 s after the press: the 19f8311b event, where the demo used to heal). The HUD frames (`BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05` takes the test spot's letterbox off; 60 fps) show the box's count 3 on the frames at 1.483 and 1.500, 2 from 1.517 (the press frame), the health bar at 60/115 to 1.667 and full in one step on 1.683 (+0.167 s), the box then grey and red (full health), and the medkit in the hand from the reach event. **An interruption is the demo's guess:** the count drops at the press as take13 shows, but a use that ends before 0a6e8f79 (a knock-down, a death, a hand-over to another member) leaves no heal and the medkit gone, since the demo ends the use clip and nothing gives it back. The game's pending holder (inv+0xf0) isn't traced on a cancel, so what it does isn't known. On a hand-over the swapped-out member's `item_used` is never consumed (`medkit_used` reads only the controlled player).
  * The medkit's cross glows: the model's second piece is a flat quad over the painted cross
    with the untextured glow material h_031724e6 (shader h_f539fe8c, glow 0.10 0.34 0.83,
    wrapper opacity 50). Flat untextured glows like it are drawn at full strength (the
    opacity is for the see-through glass shells round other pickups), as in the png.
  * Pickups are loose: every inventory-object without an idle effect, and the used medkits.
    * **At the start** each sits in the level's pose, tilted to the ground under it (up to
      ~25 degrees; steeper is a step, so it lies flat). It rests on its lowest point, not its
      origin: a model's origin can be its middle.
    * **A walk moves nothing** (#356). A character who walks into a pickup he can't take (a box,
      pack or OrgSen on a full stack, a lying gun, a used medkit, a fruit at full health, a
      squadmate's refusal) walks straight through it: nothing is nudged, hopped or held off.
      The game has no kick. Until #356 the demo had one (#91: out ahead of the character,
      faster than he went, off to the side with a hop and a roll, once per 0.6 s); with the
      walk-over's reach (1.34 m) wider than the kick's (0.45 m), the only pickups it ever
      kicked were these.
      * *The code.* The walk-over is a distance rule with no contact: FUN_0011c540 asks the
        world's cells within 2.0 m for items (19), grenades (24) and weapons (20) and hands each
        to FUN_0011c170's reach test ([code-map.md](../code-map/README.md)). The pushes the game makes on a
        loose body are FUN_002b3bc0's callers (a thrown grenade's launch, a bolt's shove, the
        ragdoll's start), none of them the character's movement.
      * *The physics' pairing.* Whether a pickup's body meets the character's capsule at all is
        Ipion's pair rule (FUN_00169a30): a lying box, pack, medkit, fruit or canister pairs
        with it (the type's h_ecb577b4 is false; the carried Medkit and a few inventory-only
        items have it set), a lying gun (kind 20) never does. What the pairing is for isn't
        read, and the footage shows it has no effect on a walk.
      * *The takes.* #18 take05 (sdm_e34, the Gas stack at 10): Tex walks 2.7 m through three
        Gas packs, two of them at his left foot, and stops among them; frames at 3.6, 4.0, 4.5,
        5.0, 6.0 and 8.0 s show the packs where they lay at 2.0 s, "Gas Grenades (full)" red the
        whole time. #61 take04: Tex walks onto the drop (the L-Shot-50 and the Jax-iP, "Tex
        cannot pick up Jax-iP." at 3.3 s) and stands there, the blue ring on the L-Shot-50 at
        his feet on the same ground point from 4.0 to 6.4 s, and after the swap (7.0-8.0 s) the
        Jax-iP is still at his feet with the red ring: neither gun is moved. (For a lying gun
        there's a second reason: a weapon never pairs with the capsule, above.) The ammo boxes
        carry the packs' collision flags (mass 100, h_ecb577b4 false), so the packs' rule is
        theirs: #18 take02's two walks onto the full Particle box stopped 0.65 and 0.80 m short
        of it and don't show it moving, and no take walks through a box (the ticket's shot S1,
        not filmed).
    * **Blasts.** A grenade blast within 1.5x its radius throws it (9 m/s at the middle, less
      toward the reach), rolling the way it's sent. The demo's choice, #91's; unchanged. **The
      turn it gets has a per-body jitter taken from the entity's index** (`physics`: `sin(index x
      12.9898)`), so what is allocated before the rack decides how a blast turns each pickup, and
      every build agrees with itself but not with another that allocates differently. A system added
      to `OnEnter(Playing)` reorders the unordered ones there and moved the rack's indices by 3:
      #211's bottle loader did, and the Orbital Beam's, the Air Strike's and the Wrath's debris flew
      differently from main's until it was made a lazy `Update` system (play_molotov_mutant.rs
      `load_bottle`). A follow-up is a jitter keyed by something that survives such changes (the
      pickup's spot); it would change every build's debris once.
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
    * **Verified, no kick** (#356; `BF_PICKUP_LOG=1`, 15 fps captures, `BF_MUTE=1
      BF_BACKGROUND=1`; the same runs on the build before the change, which logged "kicked ..."
      and a second "came to rest" for what it kicked, are quoted):
      * #18 take05's walk (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_INV=Frag:0,Gas:10
        BF_TEST_GOTO=-77.52,15.19,-78.3,11.4 BF_TEST_WALK=1 BF_TEST_GO_AT=2.0`, 150 frames):
        "the stack is full, it stays" for the three packs at 3.47, 3.60 and 4.07 s, no kick and
        no pack coming to rest anywhere (before: "kicked 1999v1 at [-78.38, 29.15, 11.69] by
        character 0 going 0.6 m/s", at rest 0.6 m off at 5.27 s). Frames 54-120 (3.6-8.0 s) have
        the packs where they lay and "Gas Grenades (full)" red, as take05's.
      * #61 take04's recreation (`--test BF_TEST_GOTO=0,0,0,-3.0 BF_TEST_GUN="L-Shot-50,3.6,0.3;
        Jax-iP,3.3,-0.35" BF_TEST_WEAPON="MK-ASLT;Confed LZR-50"`, 120 frames): both guns rest
        where they landed, (0.28, -3.60) and (-0.35, -3.30), and nothing moves them after Tex is
        0.3 m from the Jax-iP (before: kicked to (-1.16, -5.85), at rest at 2.87 s); with the
        goal at -7.0 he walks through both and nothing is kicked. (To see take04's swap ring too,
        use `BF_TEST_GOTO=0,0,0,-4.0 BF_TEST_WALK=1 BF_TEST_GO_AT=1.5 BF_TEST_SWAP=6.0`: with -3.0
        and no `BF_TEST_WALK` Tex overshoots and the camera turns away; the swap recipes under
        "Verified" below all walk.)
      * the box on a full stack (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_WEAPON="Saryl-99;Confed
        LZR-23" BF_TEST_AMMO=4:200 BF_TEST_GOTO=-51.70,82.10,-57.9,80.6 BF_TEST_WALK=1
        BF_TEST_GO_AT=2.0`, 150 frames): he walks through the box's spot (-55.98, 29.01, 81.09),
        "Particle Ammo (full)" at 4.00 s, no kick (before: kicked to (-57.36, 29.18, 80.68)).
      * a used medkit (`--test BF_TEST_HEALTH=50 BF_TEST_INV=Medkit:3 BF_TEST_ITEM=Medkit
        BF_TEST_MEDKIT=1.5 BF_TEST_GOTO=0,0,-3.1,-5.9 BF_TEST_GO_AT=4.0`, 150 frames): it falls
        to rest at (-0.82, -1.00, -0.79) at 3.53 s and Tex's line passes 0.36 m from it, inside
        the old kick's reach, with no second rest.
      * still moved: a thrown Frag's blast (`--test BF_TEST_GOTO=0,0,0,0 BF_TEST_THROW=1.0,0.6`,
        the blast at 4.47 s, 24 m out) sets the grid's pickups tumbling again (second rests
        logged at 6.8-9.3 s), and a shot (`BF_TEST_GOTO=0,0,0,0 BF_TEST_FIRE=hip
        BF_CAMERA_AIM_AT=0.71,-1.0,-14.79`) knocks the one it passes ("shot struck 363v1 at
        [0.77, -1.01, -14.54]", at rest (0.71, -14.79) before and (1.15, -18.98) at 3.0 s).
      * still taken: the packs with room (`BF_TEST_INV=Frag:0`) at 3.47, 3.60 and 4.07 s, "3x"
        to 9 Gas; the tutorial's medkits (`BF_MAP=tutorial BF_TEST_HEALTH=50
        BF_TEST_GOTO=51,14,51,20`) at 1.40 and 1.47 s.
  * The medkit sound is h_1538baad, the Sound whose file is h_15331992 (named "92193315":
    those bytes as stored in sounds-<level>.xmb), the same in all 54 levels' banks, 1.5 s.
    When the heal adds health the character also asks the chatter gate for their "healed" line
    (line_tag 12c35d69; 0x146f40, [chatter.md](../chatter.md)): its block's `probability` 10, its
    `replay-delay` 30 s (the 30 this page read as the chance before #155), priority 1 (so a
    squadmate over 15 m away never says it) and its variants by the enemies the healer fights
    (Tex and Brutus have one out of a fight, Hawk none, Flint two). In todo/medic + intenvory use
    case.mp4 Tex says e707f108 at his use and Brutus nothing at his. `BF_TEST_HEALED=1` passes
    its roll.
  * Each pickup adds a line below the middle of the screen (FUN_000ad790, one of the HUD's
    three, #314: see "The pickup lines' layout" below): the item's display name (h_1dd3a49d; its stringtable-name without one) alone for the
    first, "<n>x <name>" (the format at 0x40d734) once more of the same are taken while it's up;
    each pickup puts its time back to 4 s (the widget's +0x64, set by its constructor
    FUN_000ad690, #228; it was 3 s). take08's two medkits: "Medkit", then "2x Medkit" for 4.25 s
    after the second; take12's DNA line 295.4-300.0 s, fading out at its end. In the demo
    (`BF_TEST_GOTO=51,14,51,20`) Tex takes the tutorial's two at (50.6, 17.2) and (51.0, 17.5) on
    the same frame, so "2x Medkit" comes at once; the DNA canister's line (one) is its name alone.
    A pickup that took nothing, its stack full, reads "<name> (full)" in red (FUN_000ad790's
    count-0 form; a lying gun walked over, "Weapon pickups" below).
    Status messages ("No need to heal") show above them. They stay put on the screen (the
    capture has them by the character).
  * **The pickup lines' layout** (#314; `play_hud.rs` `update_feed`, `play_pickups.rs`
    `PickupFeed`). The widget FUN_000ad690 (HUD name 0x1eeaa123) has three lines, each the 14-point
    body font (`BODY14`: its constructor sets the record 0xc626fd5 at 0xe), centred on x 320, line
    k 20 units under the first (its update 0xad860: y = k x 0x14), the colour FUN_000b87a0's
    c0 d7 ff (192, 215, 255; take12 measures (201, 215, 252)), "(full)" FUN_000b8820's a0 20 20.
    The slot a new item takes is FUN_000ad5f0's: the first with no hold left (it may still be
    fading), else the one with the least; the same item joins the line already showing it
    (FUN_000ad650). Each line's alpha eases to 1 in 0.1 s (FUN_000ad790's call of FUN_000bbea0
    with 0.1), its 4.0 s hold runs while it is at 1, and then it fades out over **0.5 s**, the
    widget's +0x50 (FUN_000bbef0 with -1 in the base update FUN_000bbf40); an expired slot's
    item and count are cleared (0xad877), so it can't take a count from the line that was in it.
    Measured on take12 (`todo/28-tutorial`, 276-282 s): one line alone ("Frag Grenades (full)")
    has its ink 346.7 units from the top, the second slot's ("31x High ROF Ammo") 366.7 and
    its capitals 20 px (13.3 units) tall at 1.5 px a unit, centred at x 319.3-320; the bright
    pixels of a line fade over about 0.3 s (281.48-281.78 s), what a 0.5 s ramp gives (a 0.1 s one
    would be done in 0.05 s). **The ticket said 0.1 s for the fade-out and 367 for a single
    line: 367 is the second slot, and 0.1 s is the fade-in.** The first line's place (`FEED_TOP`,
    341.5 for the record text's top) is a fit to those ink rows: the widget's layout data isn't
    found. The old layout (`FEED_FIRST` 29 under the message, a 16.5 step, 11.5 pt text, no
    fade) is gone. Checked with `BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6
    BF_TEST_POWERUP="brute@0.5;force@2.0"` (120 frames): the capitals' ink rows 520-541 (slot 0)
    and 551-571 (slot 1) at 1280 x 720, take12's 520 and 551; the line out of frame 0.47 s into
    0.53 s, the first one's bright pixels falling from 4.67 to 5.0 s (4.6 s hold end + 0.5 s).
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
    spawn-triggers" (#278). **The test map has its own way back** (the demo's, not the game's
    rule): its pickup grid and the pickups put down with its object tool can only be taken once,
    so **Home** puts back every one taken, where it was and as it was put
    (`play_testpickups.rs`; [testtools.md](../testtools/putting-pickups-back.md), "Putting pickups back";
    `BF_TEST_RESET_PICKUPS=<s>`). A level's own pickups never come back, on a level the test
    session switched to included.
  * **The walk-over's reach** (#18): every pickup is taken by the same FUN_0011c540 ->
    FUN_0011c170, every living member of a player's squad against it each frame, the AI's too:
    the pickup at most 1.8 m below and 1.0 m above the character's position (its pelvis) and
    dx^2 + dz^2 under 1.8 (1.34 m across): `powerups::in_reach` for all of them. The medkits and
    fruit used to take 1.0 m across and 1.5 up or down, the DNA canister 1.3 m.
  * Test hooks: `BF_PICKUP_LOG=1` lists the pickups and each one taken or used.
    `BF_TEST_RESET_PICKUPS=<s>[;<s>...]` presses the test map's Home (the taken pickups back).
    `BF_TEST_HEALTH=<hp>` starts hurt (above the most, with the most raised to it: #152's lava
    runs at 5000, as #146's take03). `BF_TEST_MEDKIT=<s>` uses a medkit at that time. `BF_TEST_ITEM_LIST=1` holds Tab from the start (the item list open, the game paused).
    `BF_TEST_ITEM=<label>` selects a carried item at the start and `BF_TEST_ITEM_USE=<s>` presses
    G then (play.md "Items: the squad's starting inventory"). Verified (#281): e34's range spot
    with `BF_TEST_HEALTH=50 BF_TEST_ITEM=Medkit BF_TEST_MEDKIT=1.5`: "Medkit 6" blue, used,
    "Medkit 5" red at 115/115.
    `BF_TEST_CHIP=<x>,<z>[,<s>]` drops a memory chip there (see "Memory chip" in squad.md).
