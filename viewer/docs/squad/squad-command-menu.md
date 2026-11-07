*Part of [squad/](README.md), the former page `squad.md`.*

* **The Squad Command menu** (#319, the second slice of #33: `play_squad_menu.rs`; the radar's
  part in `play_hud.rs`, play.md "Radar"; todo/33-squad-commands leads §2 and §4, takes take01-03,
  take08, take09; every function in [code-map.md](../code-map/hud.md), "HUD"). How the player gives the
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
    picture and the HUD under it by (1 - k) x 175 + 80 (80 / 255 open: the console's multiply of
    the stored values, drawn as a dark grey node at alpha 0.91 when open over Bevy's linear blend:
    `backdrop_colour`, below), the title and the rows come in at
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
    campaign.md). Its input runs before the tutorial panels' (`menu_input` is ordered before
    `panels::input`, #348: the pad's A and Y are its orders and a panel's buttons both, so the
    menu's or the cursor's press is never the panel's too), and it takes nothing while another menu
    holds the game's clock (a panel's buttons are the panel's: the pad's A would otherwise be eaten
    by the cursor and a panel opened under it couldn't be closed from a pad), nor, for the
    cursor's own buttons, in a cutscene (the cursor's rows are hidden and it reads nothing then:
    Escape frees the mouse as everywhere else in one, `mouse_capture`, which the cursor had been
    eating; `takes`). Under another menu it also **reads none of the frame's buttons** into
    `MenuInput` (`read_buttons`; #348's review): the first version left them in, and the pad's A
    that closes a panel (`panels::input` runs right after the menu's input and lets go of the
    clock) was then the Move To cursor's accept too, in `squad_menu` the same frame, which
    found the clock free: one press giving two things. Test `a_press_under_a_panel_is_not_the_menus`.
  * **The order line** (#320; FUN_000c1630 after FUN_000c1280, `play_chatter.rs` `order_line`,
    [chatter.md](../chatter.md) "Lines of several parts"): for Stand Ground, Fire At Will, Cover Me
    and Move To (the cursor's accept too), whenever any selected member was handled, whether its
    mode changed or not (Cover Me to one already in FOLLOW still says it), the controlled
    character says one line, its parts one after another: the members' names, or "everyone"
    when he's ordered every other member (two or more), the order's phrase, then each member's
    answer. The members go in the squad list's slot order (Tex, Flint, Brutus, Hawk), not the order
    they were selected in; the giver's own selection adds no name or answer. The game's clock is
    held while the menu fades out, so the first sound comes with the first running frame.
    Verified against #36 take12 and #33 take03, take05, take06 and take08 (chatter.md): the names
    (116501dc Flint, f0c93785 Brutus, f5359692 Hawk, fb962749 Tex, 181b6433 a marine), the phrases
    (Stand Ground e104fe49, Fire At Will f5041bd5, Cover Me 1cdff35c, Move To f8046684), the
    answer eaf3546f and the parts' starts to within 0.05 s of the sum of the sounds' lengths +
    0.2 s. The same line from the other squads' members (#357): the giver's type's file gives the
    phrase and the names, each member's its own answer (the Feral Colonist's "Fire At Will" to the Shaman
    and the Outcast Shaman, the Militia's to Ghent alone when McTavish is ordered too). After a Fire At Will the first member of WINGMAN's list (Flint, when the order goes to all three) says 1ecaeb98 ("on my way", 75%) at its first follow update, once per order: the game's follow state 9, a stand-in since #331 for its walk to its route point (chatter.md, "The walk line"). It is Fire At Will's alone (#343): in FOLLOW the game never reaches the state, so a mission start (e34's, the test map's) says nothing.
  * **The Move To cursor** (HUD+0x44, FUN_000bd860; the game runs meanwhile). Each frame
    (0xbdfb0) it takes the world point (`bf/level.rs` `WorldPoint`: every point element of the
    level: on e34 1015 world points and 271 others, cover, door, patrol points ...) nearest the
    crosshair's centre ray's point (`play_reticle.rs` `Reticle::point`) whose flags pass (+0x24
    bit 14 set: h_0bb771a2; bit 15 clear: h_1346678f; +0x28 bit 2, valid: set on every point by
    FUN_00059b60, then by the waypoint triggers' set-valid, the route graph's, [nav.md](../nav/graph.md),
    #334) and that projects inside the 640 x 480 (FUN_0004d630, the filter 0xbdef0);
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
    Flint's overlay draw only while the HUD isn't hidden, which is `hud::HudHidden` (#348: the
    letterbox and the editor set it where they hide the roots; play_scope_view.rs used to ask
    that every `HudRoot` be visible, which any root hidden for its own sake broke).
  * **Not done**: a dead controlled character's press switching at once (0xc809c: the demo's
    death camera hands over by itself); split screen (the small layout 0x3c8fd8, no pause, the
    radar hidden while selecting); the tutorial's gates on the d-pad (FUN_000ba320 ids 11 and 14,
    #265); Medkit's heal (#329); the texts' and entries' fades as
    the console's sRGB blend (drawn in linear light, a 0.3 s difference; the backdrop's is fitted
    for it, `backdrop_colour`: take01's picture under it is 0.29-0.30 of what it was at every
    level from a stored 24 to 128 against the code's 0.3137, the demo's now 0.30-0.31 from 48 to
    160 where black at 1 - grey^2.2 gave 0.17 at 40, 0.23 at 73 and 0.25 at 108); what the cursor's
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

  Verified for the order line and the hand-over's lines (#320; `BF_MUTE=1 BF_BACKGROUND=1
  BF_TEST_FULL_SQUAD=1 BF_CHATTER_LOG=1 BF_SOUND_LOG=1 BF_SQUAD_MENU_LOG=1`, the test map,
  captures at 15 fps; chatter.md has the rows and the comparison with the takes' game clocks):
  take12's `BF_TEST_COMMAND="2:flint:0.4;
  3:stand;10:brutus:0.4;11:fire;18:hawk:0.4;19:stand;26:hawk:0.4;27:cover"` (and on e34 with
  `BF_TEST_NO_HOSTILES=1 BF_TEST_GOTO=-83.39,-82.79,-83.39,-82.79,27 BF_TEST_GO_AT=999` and the
  times 28 s later, past its radio mutes): each order one line, Tex's name call, his phrase and
  the member's answer, 0.07 s after the order, its parts 0.5-2.5 s apart by their lengths;
  take03's (`2:hawk:0.4;2.9:brutus;3.5:flint;4.1:brutus;5:stand;11:hawk:0.4;11.9:brutus;
  12.5:flint;13.5:cover`): Flint and Hawk by name, then "everyone" for all three, the answers in
  slot order; the cursor's Move To (`2:hawk:0.4;2.5:flint:0.4;3.3:move;5:accept`
  `BF_TEST_CURSOR_AT=10,45`): both names, f8046684, both answers.
  The hand-over (`BF_TEST_COMMAND="2:hawk:0.1;6:tex:0.1;10:brutus:0.4;11:stand;17:brutus:0.1;
  21:tex:0.1"`, take13's): f415ab02 for Tex and Hawk in Cover Me, refused the third time by the
  replay delay, 0e69bfe5 for Brutus on Stand Ground; the far case (`BF_MAP=e34 BF_SCRIPT=off
  BF_TEST_NO_HOSTILES=1 BF_TEST_GOTO=-83.39,-82.79,-25,-78,27 BF_TEST_SPRINT=4
  BF_TEST_ORDER=hawk,guard@2 BF_TEST_SELECT=2,26`): Tex e635ecc6, Brutus ed05c0d7 and Flint
  e97ce7fd joined.

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
    radar back in its corner, the world moving again. The backdrop's darkness measured (#348:
    the picture under it over the same picture before the menu, per pixel, outside the menu's
    entries, rows and the radar): take01 (1.9 or 2.3 s against 4.0 s) 0.28-0.30 of the picture
    at every level from a stored 24 to 128 (the fit out = 0.27 in + 1 over the lit pixels); the
    demo (frame 28 against 60) 0.30-0.31 from 48 to 160 (0.33 near 40), where it had 0.17 at 40,
    0.20 at 58, 0.23 at 73 and 0.25 at 108; the radar's slid corner (171.6, 181.0) in take01, the
    code's (172, 181), the demo's (172.0, 180.5).
  * take03 (`"...2:hawk:0.4;2.9:brutus;3.5:flint;4.1:brutus;5:stand;11:hawk:0.4;11.9:brutus;
    12.5:flint;13.5:cover"`): sel 00001, 00011, 00111, 00101 at 2.20, 3.07, 3.67, 4.27 (take03:
    2.34, 3.04, 3.64, 4.25); Stand Ground to Flint and Hawk (GUARD, a0), Brutus untouched; Cover
    Me to all three, Brutus "FOLLOW already", Flint and Hawk back to FOLLOW a0 (take03 alike);
    the three faces orange with their brackets at 3.9 s.
  * take02 (`"...2:hawk:0.1;5.5:tex:0.1;9:hawk:0.2;12:hawk:0.35;14.5:back"`): 100 and 200 ms
    presses hand over on the release (the name, the chevrons, the cut; Tex a3 -> a0 left), 350
    ms selects Hawk (controlled by then) and opens the menu, as take02. **A tap does flash the
    chevrons in take02** (#348 asked; the red pixels of the frames beside Hawk's portrait, the
    LEFT 100 ms at 2.0 s acknowledged at 2.10: none at 2.15 s, 693 at 2.20 (the three chevrons
    whole, x 71.3-91.3), 84 at 2.25 (the outer two gone), none at 2.30; Tex's UP 100 ms at 5.5 s:
    958 at 5.70 s, the chevrons above his portrait): the press's flash, not the hand-over's
    (FUN_000c4dd0, 0.25 s long), and the demo's taps flash too (frames 30-31 and 84-85 of the
    recipe, the chevrons at Hawk's and Tex's, x 70.7-92.0 and y 314.7-330.7 against the takes'
    71.3-91.3 and 313.3-338.0 after #347's refit of `CHEVRON_GAP`).
  * take08 (`"...2:hawk:0.4;3:move;8:accept"`): the cursor on world point h_ef17a343 at (-67.8,
    28.1, -76.5), the point nearest take08's own first read; its rows on take08's to the pixel,
    the marker turning; A: Hawk MOVE_OUT to it. With `BF_TEST_CURSOR_AT=-70.8,-79.5`: take08's
    chosen point itself, h_ff113737 (flags ffff4000, as take08 read), Hawk arriving 1.7 m short (2.0 m before #335's route)
    (take08 2.00 at its first GUARD row: -72.66, -80.21 against the point -70.78, -79.53) and GUARD. With `BF_CAMERA_PITCH=1.0` (the sky): no point, the marker on Tex, A
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
