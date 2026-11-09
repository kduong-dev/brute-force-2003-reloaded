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
    reactions by its personality while the AI drives it, none while a player controls it: the
    controlled character's brain is off, FUN_00129530 with char+0x834 bit 3, and the brain's
    CBT_DAMAGED case tests it first, 0x3e528, #200; its stamina shield, refilled for the
    controlled one too, its
    ragdoll, its death line), it has no skill (V says "<name> has no special skill") but the
    Orbital Beam of the types that carry it (#200: the Confed Marine, the campaign's Confed allies, and
    the demo's Red Hand and Subversive classes, #407), the Militia's Air
    Strike (#203: [skills/air-strike.md](../skills/air-strike.md)), Gunthar Ghent's Cloak (#205: Gunthar
    Ghent himself and the demo's Terrorist class, [skills/ghents-cloak.md](../skills/ghents-cloak.md)) and Tex's Berserker on the demo's Kingman
    class (#407; [skills/other-characters-skills.md](../skills/other-characters-skills.md)), whose stamina its
    skill then refills, not the enemies' refill, and says its
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
    game's): the first six rows of the menu's list (the three class squads of #365 and #394 add three after them, below). The campaign squad (the demo's name for it:
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

    **The three class squads** (#365's request, given deathmatch classes by #394 and a third squad by
    the product owner's additions; the demo's own: the data has no such squads,
    `character_class::SQUADS`) follow the five, in the same four places and the same path. Each
    member is a **deathmatch class** ([character-types.md](../character-types.md) "Deathmatch classes
    (#394)"): the campaign type of the character with the rule of the five squads' types applied (hit
    points by tier, damage-multiplier 1, the player flag, no vulnerabilities, radar-range 60, kill
    reward 0, a HUD slot with its skull and its footsteps, the cut face where the data has one), a type
    of its own in the place "demo/terrorist_cell", "demo/kingman_posse" or "demo/mutant_horde". Each
    sits in the place its **seat** names (`character_class::Seat`: the squad's, not the class's: the
    Filthy Mutant is at the left of the Mutant Horde and at the bottom of the Kingman Posse), which is
    its type's HUD slot as the real squads' members' is (`seat`); a character seated in another
    squad's library is a type of its own with that slot, so its skull and footsteps follow the seat
    (`mp_supermutant@kingman_posse`). The boss is at the top, Tex's place (the Mutant Horde's top is the
    Molotov Mutant, #401), so the player, who starts as Tex, becomes the boss or, in the Horde, the Molotov Mutant:

    | Squad (hook names) | Brutus's place (3) | Flint's place (2) | Hawk's place (4) | Tex's place (1) |
    |---|---|---|---|---|
    | Terrorist Cell (`terrorist_cell`, `cell`, `kingman`) | Red Hand (55 hp, the demo's red face) | Terrorist (50) | Subversive (70) | Edward Kingman (95) |
    | Kingman Posse (`kingman_posse`, `posse`) | Filthy Mutant (70) | Supermutant (95) | Subversive (70) | Edward Kingman (95) |
    | Mutant Horde (`mutant_horde`, `monsters`) | Ugly Mutant (75) | Supermutant (95) | Filthy Mutant (70) | Molotov Mutant (55) |

    * **The names.** The menu lists them after the five, in that order (rows 6, 7 and 8; the campaign
      squad is 0). "Kingman Posse" is the data's cut squad name, read from the string table
      (h_0f3ad7a9); "Terrorist Cell" and "Mutant Horde" are the demo's (the product owner's choices
      of 2026-11-07, the Horde in the real squads' faction + group-noun style; the briefings call
      them "Mutants"). #365 called them Kingman's squad and Monsters: those names and the hook words
      `kingman` and `monsters` still pick the Cell and the Horde (`Preset::aliases`), so the recipes that
      name them keep working; `kingman` is the Cell's, the Posse is `posse` or `kingman posse`.
    * **The Terrorist Cell's and the Mutant Horde's rosters** are #365's (the product owner's). The
      request's "Subservient" is the data's **Subversive** (the third terrorist type, its name the
      string table's; there is no Subservient). Kingman, the Terrorist and the Subversive start from
      m03_c's types, Kingman's own level; the Red Hand from e05's (the data has Red Hands in e05, e13
      with 128 hp and the tutorial with 23): a guess, the one whose 31 hp sits beside the other two's.
      m03_c holds two copies of each of Kingman, the Terrorist and the Subversive that read the same,
      the first of each is taken. The data has **three mutant variants** besides the Supermutant:
      Filthy, Molotov and Ugly; all three are in the Horde, from e23, the Supermutant's own level. Their
      places are the rule's (the product owner's choice): the heavy fighter at the bottom (the Ugly), the
      lightest member at the left (the Filthy). **The top and right are the product owner's (#401, demo-own,
      a guess)**: the Molotov Mutant at the top (slot 1) and the Supermutant at the right (slot 2), because the
      Supermutant's portrait fits a side slot; #394 had the boss at the top and the Molotov at the right. Their
      home slots follow the seats (`mp_supermutant` is slot 2, `mp_mutant_molotov` slot 1; the Posse's
      Supermutant, also at slot 2, is still its own type `mp_supermutant@kingman_posse`). #365 had the Terrorist at the right and the Subversive
      at the left, the Filthy Mutant at the bottom and the Ugly Mutant at the left; the classes swap each
      pair. **The two troopers' seats are the product owner's (#400, demo-own, a guess)**: #394 had the
      Subversive at the right (Flint's, slot 2) and the Terrorist at the left (Hawk's, slot 4); the Cell now
      has the **Terrorist at the right (slot 2) and the Subversive at the left (slot 4)**. A class's home
      seat sets its type's slot, skull and footsteps, so the Terrorist's home slot is 2 and the
      Subversive's 4.
    * **The Kingman Posse** is the cut squad of the data, matched to its group picture h_19f9d1f0 of the
      front end's `splash_screen` library (a black-armoured trooper with a rifle, the Supermutant, a
      crouching pink mutant and Kingman in his grey suit): Kingman at the top (slot 1), the Supermutant at
      the right (slot 2), the Filthy Mutant at the bottom (slot 3) and the **Subversive** at the left (slot 4),
      hit points 95 / 95 / 70 / 70. **The data has no seats for it**: no `<squad>` entry, no library, no
      multiplayer types (the string table keeps only the name). Kingman at the top follows the real squads'
      boss-at-slot-1 rule; the rest is demo-own, the product owner's: the Supermutant beside him, the mutant
      of the picture (`mutant1.tga`'s pink bare-chested one) taken as the Filthy Mutant, **a guess**, and the
      picture's trooper taken as the Subversive (both cut select pictures, `terrorist.tga` and
      `subversive.tga`, show black-armoured troopers that look alike in the group picture; #394 had the
      Terrorist, the product owner changed it to the Subversive in #400, so **which trooper it is stays a
      guess**: its seat label says so). The Subversive there is the type `mp_subversive_marine@kingman_posse`
      (`mp_terrorist@kingman_posse` is gone). The members come from the picture, not from any roster in
      the data.
    * **Portraits, voices, skills:** each class shows its cut 32 x 32 face round the radar, in
      common's textures (`face_kingman.tga`, `face_subversive.tga`, `face_terrorist.tga`,
      `face_supermutant.tga`, and the three mutants' demo-made faces of #401, below), and the slot's skull when it dies. **The Red Hand has no face in the data, so
      the demo gives it a portrait of its own** (#400; it replaces the "?"): the Terrorist's face
      (`face_terrorist.tga`, h_0be444b1, 32 x 32) with **the helmet kept** (its top 17 rows: the grey rim, the
      dark dome and the visor, as the Terrorist has them, close to the Red Hand model's dark helmet) and
      **everything below it recoloured to the Red Hand's armour**: each texel's value (the source's shading) is
      looked up in a ramp of dark brown, a red band for the shaded sides, tan for the lit plate and light tan
      for the highlight, so it reads red-on-tan like the model's collar, shoulders and chest. Every alpha is
      kept, so the background stays transparent. The ramp's colours are the dominant ones of the Red Hand's own
      skin texture, h_e2aebd26 (e05; the colour texture of its material h_f442845f; its other textures are
      the Terrorist's and the Militia's too): a k-means of its saturated texels gave dark brown (30, 17, 8),
      maroon (86, 46, 28) / (114, 61, 49), tans (140, 116, 79) / (163, 143, 97) / (171, 146, 104), light tan
      (206, 169, 129), and its fully opaque texels the markings' strong red (146, 24, 13) / (150, 40, 28). The
      stops' values on the source's shading and the 17 helmet rows are fits by eye against the Red Hand model in
      the demo. **#402 keeps the Terrorist's skin under the visor**: the Terrorist's own portrait has skin there,
      a patch of warm mauve and pink-brown texels in the opening of his helmet (picture rows 16 - 20, columns 4 - 7:
      (74, 52, 57), (98, 76, 76), (123, 100, 95) and the lit chin (148, 125, 115), framed by the visor's blue-black
      texels and the black of the neck), which the first version recoloured as armour because it lay under the 17
      helmet rows. The first #402 try (a flat band of the Red Hand's own flesh, from the pink patch of h_e2aebd26,
      over the visor's lowest rows) read as a salmon rectangle, so the product owner asked for the Terrorist's way:
      rows 17 - 20, columns 3 - 11 (`RED_HAND_SKIN_ROWS`, `RED_HAND_SKIN_COLUMNS`) now keep **the Terrorist's own
      texels**, and the ring of texels round them is half those and half the armour's colour
      (`red_hand_skin_weight`), so the patch blends into the visor and the collar instead of ending on a box. The
      patch's extent and the half weight are fits by eye on the portrait enlarged next to the Terrorist's
      (`tools/tex_tool.py export` on common's archive, flipped upright; the texture's own rows are the picture's
      bottom first). (#400's first version turned the dark armour pure red; the product owner found it too red.) It
      is **the demo's own derived image, not the data's**: `Game::texture_rgba` makes it when the texture
      `demo_face_red_hand.tga` (`character_class::RED_HAND_FACE`) is asked for, the HUD caches it as an image
      like any face. The three mutants' portraits
      are the demo's own derived images too (#401). The data has one mutant face, `face_mutant.tga` (h_fc5f519f), shared,
      with 109 opaque pure-black texels beside the head (a block at the art's right and a corner at its left: the
      radar showed a hard black box). `character_class::mutant_face` makes each portrait when
      `Game::texture_rgba` is asked for `demo_face_mutant_filthy.tga`, `demo_face_mutant_molotov.tga` or
      `demo_face_mutant_ugly.tga` (`DERIVED_FACES`): (1) **the box is cleared**: the opaque texels of value 2 or less
      joined to the art's border by a four-connected flood fill become transparent (109 texels; a black texel
      inside the head stays); (2) **recoloured** (Filthy and Molotov): each texel keeps its place in the face's own
      shading (its rank by value among the remaining texels) and takes the skin's colour at that quantile
      (a ramp of eight quantile means of the skin texture): the Filthy's from h_1ac02ad3, the head and hands'
      skin of e23's model (materials h_ed6ca9fc and h_000f0cc9; brown), the Molotov's from h_fe8edd48 (material
      h_09083c5f: the hair and fur, tan and cream); the Ugly keeps the original colours; (3) **fitted** (the second version, after the
      product owner found the first, which grew the art 32 / 27, cropped): the cleared art's whole opaque bounding
      box (27 x 21 at the top left, touching the texture's top and right edges as the data has it) is scaled **down**
      to fit a 24 x 24 box (aspect kept, never enlarged; a 0.889 scale here), centred across the texture and 3 rows below its top,
      bilinear, so nothing touches the texture's border and the whole head shows. The box is measured on the nine
      other portraits (opaque bounds): their art is 21 - 27 columns wide (mean 23.4 -> 24), 24 - 31 rows tall, from
      row 0 (Kingman's from row 3: the top margin) down to row 26 - 30 (the box's 3 - 26). The numbers are a fit by
      eye on those (`MUTANT_FIT_WIDTH`, `_HEIGHT`, `_TOP`). The portrait box is drawn whole at every slot
      (a grid texture shows all 32 x 32 texels at the top, right, bottom and left: the radar frame doesn't cover it), so
      the ring's frame hides nothing; the art's own top and right edge cut (the data's face is cropped there) is
      now inside the box, with clear padding round it. **The Filthy and Molotov Mutants are the same
      model with the same textures in the data** (compound h_18f5f8dc, materials h_09083c5f, h_ed6ca9fc,
      h_000f0cc9, h_1b065d73, in e23, e01 and m03_a alike; they differ in their AI and skills only), so
      "each its own skin" has no data answer: using two different textures of that one model to make the
      pair read apart is **the demo's choice, a guess**. The Ugly's model has its own mesh h_0544f25a
      but the same two textures h_1ac02ad3 and h_1cbbb846. The ramps, the 2 and the fit's box are labelled in
      `character_class.rs`. `BF_DUMP_FACES=<dir>` (bf_play) writes the four derived portraits as raw RGBA. The voices are the campaign types'
      (terrorist_chatter, edward_kingman_chatter, mutant_chatter, mutant_super_chatter, #357's rule; no
      sdm chatter library exists for them). The skill block is the campaign type's, kept (the
      Supermutant's CSKILL_BOSS_SUPERMUTANT 17, the Molotov Mutant's CSKILL_MOLOTOV_MUTANT 20, the
      Ugly and Filthy Mutants none); the ground pound is (#212: V plays the 1.97 s `Sc_w1_ground_pound`
      and its burst, [skills/ground-pound.md](../skills/ground-pound.md)), and **the Molotov Mutant's is** (#211, the product owner's design: V lights a
      bottle in his left hand, the HUD says "Self-Destruct", V again puts it away, a hostile within 5 m has him lunge and
      burst: [skills/molotov-mutant.md](../skills/molotov-mutant.md); his class sets the stamina update the campaign type
      lacks, so the bottle drains 3 a second, and so does a controlled campaign type's: `Skill::new` sets the flag on the
      character's own copy; control passing to another member while he holds the bottle puts it away). **Four classes carry a
      skill of the demo's own** (#407 and #205, the product owner's requests, labelled demo-own guess in their `why`
      rows: `Class::skill`): the **Terrorist** Gunthar Ghent's **Cloak** (CSKILL_CLOAK 8, cost 6, recovery
      1.5, max-stamina 100, no sounds; V or `BF_TEST_SKILL`, the HUD says "Cloak", he fades into the
      refraction as Hawk does and a shot costs the gun's damage: [skills/ghents-cloak.md](../skills/ghents-cloak.md);
      Flint's place, `BF_CHARACTER=flint BF_TEST_SQUAD_PRESET="0.3,terrorist_cell"`), the **Red Hand** and the
      **Subversive** the Confed Marine's **Orbital
      Beam** (CSKILL_ORBITAL_BEAM 9, cost 55, recovery 2.3, max-stamina 100; V or `BF_TEST_SKILL`, the
      HUD says "Orbital Beam", the beam lands: [skills/other-characters-skills.md](../skills/other-characters-skills.md)),
      and **Edward Kingman** Tex's **Berserker** (CSKILL_TEX 3, cost 6.5, recovery 1.2, the deathmatch
      Tex's, objecttypes-dm.xml line 51, in place of his CSKILL_KINGMAN 5 whose briefcase #201 isn't built). Kingman also starts with
      **two pistols**: **two Kman Autos**, both on his High ROF Ammo (demo-own, `Class::extra_gun`; the inventory holds the
      two of one weapon: the HUD lists both, the switch swaps them and a reload takes from the one reserve,
      against his data: his `<limit-weapon one="0 8" two="8 8">` gives slot 2 no class, and the demo's
      `spawn_unit` doesn't check it; switched between as usual, no dual wield). Since #410 each Kman Auto
      belongs to a side and its hand, the first on his right hip and in his right hand, the second on his left
      hip and in his left hand (`play_holster.rs`), and with the left one in hand he aims and fires left-handed
      (his arms exchanged, `play_mirror_arm.rs`). In Berserker both are drawn, each by its own hand from its own
      hip, and the arm without the gun in hand is **posed as the mirror of the other** (demo-own: only Tex's model
      has a SECOND_HAND and berserk clips), so it aims and kicks as the other does, and both fire; hanging,
      walking and running the arms are plain mirrors across his torso, aiming across the aim's plane (#409). His
      hands draw and holster (`play_reach.rs`, #410): the swap goes side to side, the hand in use putting its gun
      on its own hip and the other drawing from the other hip. The Terrorist Cell's bottom seat is the
      Red Hand: `BF_CHARACTER=brutus BF_TEST_SQUAD_PRESET="0.3,terrorist_cell" BF_TEST_SKILL=2.5` fires the
      beam; `BF_CHARACTER=hawk` is the Subversive, the default Tex's place Kingman. A class has the sdm types' common camera, not the
      campaign type's 1.55 m (the body filled the lower 40% of the screen): 3.2 m at 1 degree, every class, by
      the rule (the sdm Militia's block for the militia-model terrorists, the same set for the bosses and the
      mutants, whom no sdm type matches; the product owner chose it for the two bosses over the sdm Hunter
      Lord's 4 m at 5); [character-types.md](../character-types.md).
    * **The log.** `BF_TOOLS_LOG=1` lists each class squad with its members and, under each, the
      class's values and where each came from, real / rule / guess (`Class::report`: each value of the
      class, the seat's slot, the rule's columns); `BF_TESTMAP_LOG` says which of the twelve class types
      were made and why one wasn't.
    * **On a squad deathmatch map** (`BF_MAP=sdm_e34`) the squad menu and `BF_TEST_SQUAD_PRESET` need
      `--test`: they are part of a test session, so `--test` with `BF_MAP=sdm_e34 BF_TEST_SQUAD_PRESET=...`.

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
    window shows the last six of the nine squads above it, since #365 and #394 added three; the first
    three are a scroll up away), and, with a squad picked, who goes in each place, two
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
