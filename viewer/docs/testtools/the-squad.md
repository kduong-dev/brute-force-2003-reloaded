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
    game's): the first six rows of the menu's list (#365 adds two after them, below). The campaign squad (the demo's name for it:
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
    library or type goes with it, so it isn't offered.)

    **The two enemy squads** (#365; the product owner's request, the demo's own: the data has no
    such squad, `testsquad::ENEMY_SQUADS`) follow the five, in the same four places and the same
    path. Each member is found by name and level (`Game::find_character_type`, "name@archive"), one
    to a slot in the keys' order (Brutus's, Flint's, Hawk's, Tex's), so the player, who starts as
    Tex, becomes the last:

    | Squad (hook names) | Brutus's place | Flint's place | Hawk's place | Tex's place |
    |---|---|---|---|---|
    | Kingman's squad (`kingman`) | Red Hand (e05, 31 hp) | Terrorist (m03_c, 25 hp) | Subversive (m03_c, 36 hp) | Edward Kingman (m03_c, 311 hp) |
    | Monsters (`monsters`) | Filthy Mutant (e23, 55 hp) | Molotov Mutant (e23, 55 hp) | Ugly Mutant (e23, 55 hp) | Supermutant (e23, 669 hp) |

    * The request's "Subservient" is the data's **Subversive** (the third terrorist type, its name
      the string table's; there is no Subservient). Kingman, the Terrorist and the Subversive are
      m03_c's, Kingman's own level; the Red Hand is e05's (the data has Red Hands in e05, e13 with 128
      hp and the tutorial with 23): a guess, the one whose 31 hp sits beside the other two's.
      m03_c holds two copies of each of Kingman, the Terrorist and the Subversive that read the same
      (their keys differ: the menu adds the key), the first of each is taken.
    * The data has **three mutant variants** besides the Supermutant: Filthy, Molotov and Ugly. All
      three are in the squad, from e23, the Supermutant's own level (their other copies: e01's and
      m03_a's Filthy, e01's Molotov and Ugly, e05's Filthy; e23's have 55 hp, the others 22-30), so the
      four come from one archive. Nothing was left out to choose.
    * **Portraits, voices, skills:** none of the eight has a portrait (`CharacterType::portrait` is
      empty, as for every campaign enemy: only the squads', the deathmatch types' and a few marines'
      have one), so each radar place shows the "?" (#342's rule). They have voices: terrorist_chatter
      (Red Hand, Terrorist, Subversive), edward_kingman_chatter, mutant_chatter (the three mutants) and
      mutant_super_chatter (the Supermutant), by #357's rule (their lines were not listened to here). None has a skill: Kingman's
      briefcase (#201) and the Supermutant's ground pound (#212) aren't built, and V says "has no
      special skill". Controlled, Kingman and the Filthy Mutant move by their own clips (seen
      running with the follow camera; the Supermutant's and the others' were not driven).

    **Choosing one** (Enter or a click on its
    row, `BF_TEST_SQUAD_PRESET`) puts its four members in their places one after the other in the
    keys' order (Brutus's, Flint's, Hawk's, Tex's), each by the single-slot path above (`change`): a
    fresh character of its type, your own slot's in place where you stand (its view, clock and the
    squad's inventory kept), an empty slot's at its formation place round you, a member already
    there replaced where it stands. "Squad: Shrike Raiders (you're Hunter Lord now)" shows; on a map
    played alone (an arena) only your slot changes ("3 not put: played alone"); in edit
    mode only the roster ("(when play starts)"). **The panel** (`testsquad::panel_text`) shows
    seven rows of the list (`ROWS`) in a window that stays put while the pick moves inside it and
    scrolls a row at a time past its edges (it opens on the top of the list: the six squads in view
    and the first type, picked, at its foot), and, with a squad picked, who goes in each place, two
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
