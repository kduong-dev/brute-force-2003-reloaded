*Part of [testtools/](README.md), the former page `testtools.md`.*

* **The squad** (`play_testsquad.rs`, #302; the product owner's choices, not the game's: its squad
  is always Brutus, Flint, Hawk and Tex, each in its own place):
  * `BF_CHARACTER` (play.rs `start_character`, #330) picks that character, on the test map and on
    every level: a squad name (`brutus`, `flint`, `hawk`, `tex`, any case) or its slot number
    (0-3: Brutus, Flint, Hawk, Tex); Tex when unset or when the value is neither (the demo says so
    on stderr). Before #330 only the numbers parsed, so a name silently gave Tex. Checked on the
    test map (`--test BF_TOOLS_LOG=1`, the "squad at the start" line) for `flint`, `1`, `Hawk`,
    `brutus`, `tex` and `0`.
  * The test map starts with the player's character alone (Tex, or `BF_CHARACTER`'s), its
    other three slots empty, on every start of the flat floor (`--test`, `--edit`, a level
    switch back to it). Any other map keeps its own squad (the four, or a level's squad spawns),
    and so does a session started on a level (`--test BF_MAP=<level>`).
  * A slot is one of the squad's four places (`Player::character`): its portrait's place round
    the radar (Brutus's bottom, Flint's right, Hawk's left, Tex's top), its health channel, its
    speech icon and the key (1-4) that takes control of it. The menu fills it with any type, as
    a real squad member, not an NPC: it gets a formation place (FORMATION by its place in
    `Squad`, where the members come before the NPCs: where it's put, the follow being the game's
    since #331, [squad.md](../squad/the-follow.md) "The follow") and the squad AI, control passes to it (1-4,
    `BF_TEST_SELECT`, the death camera's hand-over: the first living member in `Squad`'s order),
    the HUD shows it, it takes the squad's pickups and memory chips and drops a chip when it
    dies, and the controls panel names it. A new member appears at half its formation place's
    spacing round the leader, facing the leader's way; one replaced keeps the old one's place
    (a dead one's where it stood), facing its way.
  * Its data is its type's (play.rs `Player::data_slot`; the main game's squad members are their
    slots' types, so nothing changes there): a squad type in another's place is still itself
    (its skill, voice lines, camera, scope, Brutus not kneeling, its own face on the HUD at that
    place). Any other type is "a type of its own", as an NPC of it: its model, clips, HP, weapons,
    flesh, decals and damage are its type's; its behaviour is play_enemies.rs's (its hit
    reactions by its personality, its stamina shield, refilled for the controlled one too, its
    ragdoll, its death line), it has no skill (V says "<name> has no special skill") and says its
    type's chatter lines (#357: as a member of the squad every line it's asked, the order line,
    the hand-over's, the hurt, kill and death lines, from its type's `chatter-library`; as an NPC
    its hurt lines and death cry only), and its follow camera and death camera framing are its type's
    camera block (`CharacterType::camera`, FUN_0012eca0: sdm_shrike's Shrike Heavy 3.2 m at 1
    degree, its Hunter Lord 4 m at 5, the campaign's Shrikes 1.55 m at 10, so close that the
    body turns see-through with the camera's fade). Its portrait is its type's
    (`CharacterType::portrait`, h_f9d94425: the
    deathmatch types and some of the militia have one, common's textures), else a "?" in pale
    blue; its skull its type's (h_00c51907, #342), and none (a dead one's place goes blank, as the game's
    widget hides itself) where the type has none: the tutorial's marines, which have a portrait.
  * An empty slot shows no face, command tab or channel fill, on the test map and on any
    level alike (#303: the main game's rule for a slot with no member in the world, play.md
    "Radar", "The portraits"; before it the test map's alone); its channel's frame outline is
    drawn all the same, as the game's channel draw does (#354). A dead member keeps its skull, drawn in the same 32 x 32 box as every portrait (play.md, "The portrait box and the skulls").
  * **Dying alone** on the test map respawns the player (`testsquad::respawn`; the product owner's
    rule for the test map, not the game's, where a lone death ends the mission): with nobody to
    hand over to (no living squad member in the world, no hand-over under way), RESPAWN_TIME (4
    s, the death camera's time to its hand-over) after the death it comes back where it stood
    when it died, facing its way, a fresh one of its type (the menu's own-slot change: its body
    taken away, its view, the hooks' clock and the squad's inventory kept; the guns, blood and
    memory chip it left stay), and "Respawned" shows. The dead character's own clock stops, so
    the wait counts the hooks' clock on: hooks due during it or after run on time. Only on the
    test map's floor (a level a test session loads keeps the main game's rule: the camera stays on
    the body), never for the editor's stand-in. `BF_TOOLS_LOG` prints "t 5.13: tex died alone:
    respawned after 4 s at [-0.00, -1.05, 8.00], its heading -0 deg". Respawned beside the memory
    chip it dropped (0.5 m ahead), it takes it at once.
  * The controlled character's slot changes its type in place: a fresh character of the new
    type where it stood, facing its way, with its view (the aim, the follow camera easing to the
    new boom), the hooks' clock, the help panel and the squad's inventory (its grenades refilled
    to the start's as its body is made, as at a respawn). A dead one comes back alive. It can't be
    emptied. Any other change makes a fresh character too: full health, its type's guns.
  * Removing or adding a member moves `Squad`'s places, so the shots on their way are dropped
    (`AiHits`, as the NPC tool's removal does). In deathmatch (an arena, `BF_NO_SQUAD`) the map is
    played alone: the menu changes only the player's type, and refuses a member ("No squad here:
    the map is played alone"), in the editor's edit mode too.
  * **The predefined squads** (#339; `testsquad::presets`; the product owner's request, the data the
    game's): the first six rows of the menu's list (the two class squads of #365 and #394 add two after them, below). The campaign squad (the demo's name for it:
    common's Brutus, Flint, Hawk and Tex, each in its own place), then the five squads squad
    deathmatch offers, from game-options' character list h_f5a42d61: its `<squad name image
    h_fed0689a library-name>` entries (`Game::option_squads`; the parser FUN_00198d50), in the
    file's order and by the game's own names (its string table). Each member is one of the list's
    `<character>`s (`archetype-name`, the deathmatch library's type, e.g. `mp_tex`), whose
    h_fa8c572a names its type in the squad's own library (`GameCharacter::squad_type`: `mp_tex` is
    sdm_bf's Tex h_fa6ca831). That holds for all 20 characters (checked against the objecttypes:
    the two types have the same motion set, HUD slot and hit points; the Flint place's McTavish's
    portrait texture differs), and each library `mp_common/objecttypes-sdm_*.xmb` holds exactly the
    four types its squad's members name; a match loads the squad's library (FUN_000d5f60). **The
    place** each takes is its type's h_072ab4af, the HUD slot (FUN_000c84e0, FUN_000c7cf0: 1 Tex's
    place at the top of the radar, 2 Flint's at the right, 3 Brutus's at the bottom, 4 Hawk's at the
    left; the d-pad and the Squad Command menu go by it, [squad.md](../squad/README.md)): each squad's four
    members have the four slots once each. The file's member order isn't the place order (Brute
    Force lists Tex, Brutus, Hawk, Flint; Seer Coven the Fire Hound, Priest, Follower, Shadoon).

    | Squad (library) | Brutus's place | Flint's place | Hawk's place | Tex's place |
    |---|---|---|---|---|
    | Campaign squad (common) | Brutus | Flint | Hawk | Tex |
    | Brute Force (mp_common/sdm_bf) | Brutus (75 hp) | Flint (55) | Hawk (50) | Tex (70) |
    | Feral Hunting Pack (sdm_feral) | Feral Shaman (70) | Feral Outcast (75) | Outcast Shaman (70) | Feral Colonist (75) |
    | Shrike Raiders (sdm_shrike) | Shrike Soldier (50) | Shrike Heavy (55) | Shrike Hound (45) | Hunter Lord (80) |
    | Seer Coven (sdm_seers) | Seer Follower (75) | Seer Priest (85) | Fire Hound (45) | Shadoon (95) |
    | Ferguson Platoon (sdm_ferg) | Confed Marine (80) | McTavish (80) | Gunthar Ghent (80) | Militia (80) |

    (The string table has a sixth name beside them, h_0f3ad7a9 "Kingman Posse": no `<squad>` entry,
    library or type goes with it, so the game never offers it; the demo's own squad below takes the
    name.)

    **The two class squads** (#365's request, given deathmatch classes by #394; the demo's own: the
    data has no such squads, `testsquad::CLASS_SQUADS`) follow the five, in the same four places and
    the same path. Each member is a **deathmatch class** ([character-types.md](../character-types.md)
    "Deathmatch classes (#394)"): the campaign type of the character with the rule of the five
    squads' types applied (hit points by tier, damage-multiplier 1, the player flag, no
    vulnerabilities, radar-range 60, kill reward 0, a HUD slot with its skull, the cut face where
    the data has one), a type of its own in the place "demo/kingman_posse" or "demo/mutant_horde"
    (`mp_edward_kingman` and the other keys, which `BF_TEST_NPC` and `Game::find_character_type`
    take). It sits in the place its class's HUD slot names, as the real squads' members do (`seat`),
    the boss at the top, Tex's place, so the player, who starts as Tex, becomes the boss:

    | Squad (hook names) | Brutus's place (3) | Flint's place (2) | Hawk's place (4) | Tex's place (1) |
    |---|---|---|---|---|
    | Kingman Posse (`kingman_posse`, `kingman`) | Red Hand (55 hp, no face) | Subversive (70) | Terrorist (50) | Edward Kingman (95) |
    | Mutant Horde (`mutant_horde`, `monsters`) | Ugly Mutant (75) | Molotov Mutant (55) | Filthy Mutant (70) | Supermutant (95) |

    * **The names.** "Kingman Posse" is the data's cut squad name, read from the string table
      (h_0f3ad7a9); "Mutant Horde" is the demo's (the product owner's choice of 2026-11-07, in the
      real squads' faction + group-noun style; the briefings call them "Mutants"). #365 called
      them Kingman's squad and Monsters: those names and the hook words `kingman` and `monsters` still
      pick them (`Preset::aliases`), so the recipes that name them keep working.
    * **The rosters** are #365's (the product owner's, not the cut squad picture's mix of a
      trooper, a mutant, the Supermutant and Kingman). The request's "Subservient" is the data's
      **Subversive** (the third terrorist type, its name the string table's; there is no
      Subservient). Kingman, the Terrorist and the Subversive start from m03_c's types, Kingman's own
      level; the Red Hand from e05's (the data has Red Hands in e05, e13 with 128 hp and the
      tutorial with 23): a guess, the one whose 31 hp sits beside the other two's. m03_c holds two
      copies of each of Kingman, the Terrorist and the Subversive that read the same, the first of
      each is taken. The data has **three mutant variants** besides the Supermutant: Filthy,
      Molotov and Ugly; all three are in, from e23, the Supermutant's own level.
    * **The places** are the rule's (the product owner's choice): the boss at the top, then the
      heavy fighter at the bottom, the lightest member at the left. #365 had the Terrorist at the
      right and the Subversive at the left, the Filthy Mutant at the bottom and the Ugly Mutant at
      the left; the classes swap each pair.
    * **Portraits, voices, skills:** each class shows its cut 32 x 32 face round the radar, in
      common's textures (`face_kingman.tga`, `face_subversive.tga`, `face_terrorist.tga`,
      `face_supermutant.tga`, and `face_mutant.tga` shared by the three mutants, the product
      owner's choice), and the slot's skull when it dies; the Red Hand has no face in the data, so its
      place shows the "?" (#342's rule). The voices are the campaign types' (terrorist_chatter,
      edward_kingman_chatter, mutant_chatter, mutant_super_chatter, #357's rule; no sdm chatter
      library exists for them). The skill block is the campaign type's, kept (Kingman's
      CSKILL_KINGMAN 5, the Supermutant's CSKILL_BOSS_SUPERMUTANT 17, the Molotov Mutant's
      CSKILL_MOLOTOV_MUTANT 20, the others none), but the skills themselves aren't built (the
      briefcase #201, the ground pound #212, the self-destruct #211, the corpse eater #210), so V
      says "has no special skill" for all eight.
    * **The log.** `BF_TOOLS_LOG=1` lists each class squad with its members and, under each, the
      class's values and where each came from, real / rule / guess (`Class::report`); `BF_TESTMAP_LOG`
      says which of the eight classes were made and why one wasn't.

    **Choosing one** (Enter or a click on its
    row, `BF_TEST_SQUAD_PRESET`) puts its four members in their places one after the other in the
    keys' order (Brutus's, Flint's, Hawk's, Tex's), each by the single-slot path above (`change`): a
    fresh character of its type, your own slot's in place where you stand (its view, clock and the
    squad's inventory kept), an empty slot's at its formation place round you, a member already
    there replaced where it stands. "Squad: Shrike Raiders (you're Hunter Lord now)" shows; on a map
    played alone (an arena) only your slot changes ("3 not put: played alone"); in edit
    mode only the roster ("(when play starts)"). **The panel** (`testsquad::panel_text`) shows
    seven rows of the list (`ROWS`) in a window that stays put while the pick moves inside it and
    scrolls a row at a time past its edges (it opens with the first type, picked, at its foot: the
    window shows the last six of the eight squads above it, since #365 added two; the first two
    are a scroll up away), and, with a squad picked, who goes in each place, two
    to a line in the slot rows' order ("Brutus: Shrike Soldier   Flint: Shrike Heavy" / "Hawk:
    Shrike Hound   Tex: Hunter Lord (you)"). It stays above the HUD's item box (its top at y 565 of
    720 at 1280 x 720) in every state: it starts 18% down the window, so it may take 26 lines of
    13-point text, 15.6 px each with 8 px of padding; the worst state, a filled squad of long names
    (each slot row wraps to two lines) with a squad picked, takes 24 and ends at y 520 (the tests
    count the lines of 1,728 states of the six squads' real names: the worst 24). A squad deathmatch squad's members are types of their own in the
    demo, as any type the menu puts in a slot: no skill, no squad voice (V says "has no special
    skill"), the portrait, camera, weapons, stamina shield and behaviour of their types. That
    includes Brute Force's, the sdm versions of the four (the squad deathmatch maps' own squad is
    the campaign's four with sdm_bf's skill data and damage-multiplier, `load_squad_library`); the
    campaign squad is the one with the skills and the voices.
  * The roster (`testsquad::Roster`): each map's slots as its start made them and the menu
    changed them. In the map editor's edit mode the squad is out of the world, so the menu changes
    the roster alone ("(when play starts)"), and play (F5) makes its fresh squad from it.
