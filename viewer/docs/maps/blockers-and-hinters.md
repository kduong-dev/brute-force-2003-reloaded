*Part of [maps/](README.md), the former page `maps.md`.*

  * **Blockers and hinters (#292).** A level's `<blocker>` carries a `physics-id`, the PID_ kind
    of its body (its attribute handler FUN_00197e40 reads it through the PID_ names,
    FUN_00186320): 2768 of the 2952 blockers of the 54 levels are 4, PID_BLOCKER, and 184 on 28
    levels are 6, PID_HINTER. A character's sweep meets the world's blockers by a mask of kinds
    (FUN_001525f0) that its physics step builds (FUN_0012cd10, code map): the blockers always,
    the hinters only for a character no player controls (one whose brain has +0x3c4 bit 12, set
    while its route jumps between world points, FUN_00026520, passes them too). A hinter is a wall
    the AI keeps to and the player walks through. 181 of the data's 184 hinters are open bands
    of upright faces (no floor, flank or face looking down among them), none is closed
    (`BF_FIND_BODIES` over the 28 levels, #338: #292's survey said all of them, with e01's shapes
    where a hinter's name was e01's, "Which level's shapes" above); the three others, m02_b's
    body 89 (28 steep and 28 downward faces among its 72 triangles) and m09_a's body 39 and
    m09_x's body 49 (6 floors, 16 steep, 22 downward among 48), are shaped otherwise. The demo's
    floor queries for a character the AI moves (`Body::Character`, `Mover::Ai`) take their faces
    in as any blocker's; for the player's (`Body::Player`, `Mover::Player`, #341) they leave a
    hinter out whole, as its sweep does: `Arena::floor_at`, `floor_around`, `floor_kind`,
    `flank_below` and the spawn's `floor_below` all take who asks. #338's survey of three
    bodies as "stood on like any blocker" had left the player on body 89's steep faces (28 of
    them, normal y 0.12: the flank a body falling past the band lands on and slides down:
    `flank_below`) and on m09_a's and m09_x's floors, which don't exist for the player now.
    The bands run round ground the AI shouldn't leave: sdm_e13's 2.7 km
    band (body 71, 25 m tall) traces its arena's outer lava shore, m14_c's longest (body 49,
    375 m) the edge of its icy plateau.
    * The demo made every blocker a wall to everyone. Before #285 the push-out let a body run
      through a thin band; #285's sweep holds a body at it, so the hinters became walls to the
      player: on sdm_e13 (`BF_TEST_GOTO=90,-91,130,-91`) Tex runs down to the shore and is held
      at the band at (94.9, -102.0), sliding along it, on 018e360 (#285 merged) and on main
      051dce2; on 62b6cd9 (before #285) he went through it into the lava. On m14_c (the free
      camera's teleport to (59.1, 75.4), running to (64.0, 81.7)) main held him at (61.5, 78.4),
      short of the plateau's edge.
    * Now the sweep and the step-up test (`Arena::move_body`, `Arena::clear`) take who moves
      (`arena::Mover`, from `Player::controlled`): the hinters' walls (`Tri::hinter`) are left out
      for the controlled character and kept for the AI's (the wingmen, the enemies, another
      member's cutscene walk; the controlled character's own cutscene walk stays the player's,
      whether or not the game's hold clears the player slot's flag; the demo's AI walks no routes,
      so the hinters always hold it). The player's move is swept in 4 passes, the AI's in 3, as
      FUN_0012cd10 sets them. On sdm_e13 the band holds the wingmen at the outer shore as the
      game's AI is held; the demo's own shore stop for them (squad.md, "Liquids that burn") is
      unchanged.
    * Where (`BF_FIND_HOLES`: the hinters' walls on ground the player can walk to, in metres of
      floor, an upper bound; measured again after #338, which gave the maps their own hinters
      and blockers: the numbers before in brackets where they changed): e13 2613 (18), sdm_e13
      2607 (1532), m14_a 1608 (400), m07_b 1338 (352), m02_a 1323 (670), m02_c 1223 (426), m14_b
      1065 (333), m07_a 973 (494), m04_c 963 (304), m02_b 665 (225),
      m09_a 567 (49), m14_c 523 (289), m09_x 512 (210), m09_b 489 (none), e05 366, sdm_m03 337
      (19), m04_a 308 (276), m03_b 293 (294), m03_a 246 (232), e01 212, the tutorial 198 (51), e16
      117 (31), e40 78 (16), sdm_e40 10 (11), m09_c 8 (14), m07_c none (1); m05_y's one hinter is on
      no such ground; sdm_m07's six (a level with no terrain) aren't scanned. The fix takes the
      kind, so it covers every one.
    * **Only a character meets them (#298).** Ipion's body-pair table at 0x3c01e8 (FUN_00169ab0:
      a byte per pair of PID kinds, 0 never collide, 1 always, 255 ask the pair's own check;
      code-map.md) pairs PID_BLOCKER (4) and PID_HINTER (6) with PID_CHARACTER_PHANTOM (10)
      alone, and the one world query that takes them in is a character's sweep (the weapons'
      leaves them out: play.md, "Bolts and their hits"). So the demo's other queries pass through
      every blocker and hinter (`arena::Body::Thing`): a grenade's flight and its bounces off the
      walls, a corpse's ragdoll bodies (the faces they meet and the floor under them, ragdoll.md;
      a living character knocked down keeps a character's, as the demo stands it up where its
      ragdoll's pelvis lies and the game's knockdown moves the character itself), the floors that pickups, lying guns,
      casings and memory chips settle and land on, a missile's and an instant shot's hit
      (`Arena::ray_normal`, `ray_material`, `ray_breakable`: a breakable object's own blocker no
      longer takes the shots meant for it, its collision does), the decals' surfaces and the
      blasts' ground. What a character's body meets stays as it was (`Body::Character`): its sweep
      (`Mover`), the floors it stands on, the spawns' places, the AI's ledge probe; so do the test
      tools' and the editor's spots, which put down characters and objects alike. The cameras'
      rays and a blast's line to a body (`Arena::ray`: the crosshair's, the death camera) still
      stop at a blocker: which of them the game's filters leave out isn't traced. **The AI's sight
      ray doesn't (#244, #340):** its filter, the standard line-of-sight one (FUN_00057730's mask
      0x58 with FUN_0004cfd0's 0x150: PID kinds 3, 4, 6 and 8 left out), skips the blockers, the
      hinters and the fly-blockers, so the demo casts it with `Arena::ray_shot` (the Roller's
      `roller::clear`): an out-of-bounds wall or a lava pit's hinter band hides nobody from an AI
      ([brain.md](../brain/sight-and-contacts.md) "The sight ray").
      * Before, a grenade or a Roller that ended a frame within its radius of a blocker's wall was
        pushed out and bounced back (often not: at a grenade's speed a thin band is crossed between
        two frames), and a blocker's top was a floor to a grenade, a ragdoll, a pickup and a
        decal (sdm_e34's vehicles h_fc0ef12e have their blockers' tops 0.14 m over their roofs).
      * Checked: `cargo test --lib arena` (`a_blocker_meets_characters_alone`: a blocker's and a
        hinter's top, walls and rays against a thing and a character, and a plain object's box
        against both). sdm_e13's band at the shore (`-- --test BF_MAP=sdm_e13 BF_NO_SQUAD=1
        BF_NO_HUD=1 BF_TEST_GOTO=99.64,-92.92,115.84,-104.52 BF_TEST_GO_AT=100
        BF_TEST_GRENADE_TYPE=Roller BF_TEST_THROW=1.0,0.1 BF_GRENADE_LOG=1 BF_CAPTURE_FPS=120
        BF_TEST_FREECAM=0.5,104.82,35,-87.31,21.6,-24`): the Roller set down 3 m short of the
        band rolls down the shore through it and into the lava at (105.88, -97.90) at 3.75 s;
        with the walls met (the old rule, the same run) it was pushed out at the band at 2.68 s,
        (101.78, -94.96), and rolled back up the shore. No pickup or lying gun of the 53 levels
        starts over a blocker's floor (the floor under each is the same for a character and a
        thing), so where the levels lay them is unchanged. sdm_e34's squad run
        (`BF_TEST_GOTO=6,60,6,90`, 120 frames): the same frames as before, pixel for pixel.
