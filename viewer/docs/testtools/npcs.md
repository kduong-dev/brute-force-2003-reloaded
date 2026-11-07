*Part of [testtools/](README.md), the former page `testtools.md`.*

* **NPCs** are `Player`s added at the end of the squad with `npc` set, so everything that acts
  on the squad acts on them: the player's shots (with instant kill), blasts, Sentries,
  ragdolls, blood. They aren't in the formation, can't be taken control of (1-4, the hand-over
  after a death) and don't show on the portraits, health bars or speech icons (the squad menu's
  members do: **The squad**, below). Friends are on
  the squad's team (0), enemies on `ENEMY_TEAM` (1); the hooks can put one on any of the game's
  ten teams, `neutral` (2, ALIGN_NEUTRAL) among them, and give it a radar-contact (#141).
  * An enemy sets off the squad's Sentries. The crosshair turns red on an enemy in its lock box
    (out of the box an enemy under its centre stays blue, whatever its radar-contact: a hostile
    skips the radar test), green with its centre on a friend, and yellow with its centre on a
    friend or a neutral given radar-contact 2 or 4; a neutral with its 1 is blue ([play.md](../play/crosshair-colours.md),
    "The crosshair's colours").
  * A **dummy** stands where it was put, facing the way it was put, a target for the player:
    the AI never fires at it (as BF_TEST_HOSTILE's held squadmate; the product owner's
    choice: the squad shoots only enemies set to fight). It doesn't dive from grenades.
  * **Who sees whom** is the game's team table (0x3ffd80, `damage::hostile`; #175): a character's
    sensor takes as targets only the teams its team is hostile to, so a `neutral` set to fight
    neither fires at the squad nor draws its fire (the squad's 0 is hostile to 1, 7 and 8 only).
    Checked with `BF_TEST_FULL_SQUAD=1 BF_TEST_NPC="1,brutus,neutral,fight,4.5,6" BF_SIGHT_LOG=1
    BF_COMBAT_LOG=1 -- --test` (150 frames): the NPC logs "sees no hostile" and no contact, where
    `enemy` finds the four squad members as contacts at 1.07 s and, before #175, `neutral` did too
    (31 hits on Tex in 300 frames, now none); at `4.5,-12`, in front of the squad, `enemy` is a
    contact of Brutus, Flint and Hawk and `neutral` is seen by none.
  * One set to **fight** (since #336) closes in on the nearest hostile it knows as a level
    character's attack goal does ([brain.md](../brain/goals-on-routes.md) "Its goals on routes", [nav.md](../nav/enemies-on-routes.md)
    "The enemies on routes"): its personality's EVT_ENEMY_SEEN goal (the placement's personality,
    else its type's; an NPC has no brain to hold a cover point, so its ATTACK_COVER refuses and
    is struck out and its DEBUG1 runs ATTACK_VARY, #384), and for an approaching goal
    (GOAL_ATTACK, ATTACK_VARY, DEBUG1) a dash
    without firing while the target is beyond its stand-off (its type's h_150b4e02, at most its
    gun's reach), straight across the test map's floor (no world points there; by routes on a
    map that has them), until (the near bound + 3 x the stand-off) / 4 from it: 26.25 m for most
    types. There, or for a goal that stands (the guards'), it stays put (but dives from a grenade
    landing near, as the squad does) and aims and fires at the nearest hostile in sight, and the
    squad AI fires at it. A hostile it sees within its too-close radius (3 m for the Red Hand and the
    mutants) sends it back (ATTACK_FALLBACK, #383, [brain.md](../brain/fallback.md) "The fallback"): 7 m straight
    away from it on the floor, facing and firing at it, done once it is 8 m off or at the walk's end
    (a target that follows doesn't start it again meanwhile: the event stays queued); a Filthy Mutant
    (`BF_TEST_GOTO=4.5,30,4.5,30 BF_TEST_NPC="0.5,h_0b5f6e21,enemy,fight,4.5,28"`) backs off at
    1.84 m/s (BACKSTEP x 0.8) from 2 m to 8 m in 3.7 s. A squad type (Brutus, Flint, Hawk, Tex) set to
    fight has no drive and stands as before. It has no home area and doesn't pursue. A Filthy Mutant
    (`BF_TEST_GOTO=4.5,30,4.5,30 BF_TEST_NPC="0.5,h_0b5f6e21,enemy,fight,4.5,-15" BF_TEST_HEALTH=5000
    BF_BRAIN_LOG=1 BF_GOAL_WATCH=0.5 -- --test`) dashes in from 45 m and stands 25.9 m from Tex at
    3.1 s, firing from 4.7 s. This is the squad AI's fire, which until now never had a target
    (play_brain.rs `glance`, play.rs `ai_fire`): an AI character sees by its sensor, as a level
    character's brain does (#197, [brain.md](../brain/sight-and-contacts.md) "Sight and contacts"): anyone on another
    team within its type's vision-range (its placement's `vision` when that says so) x 1.15 once
    it's a contact x their visibility (a stealthed Hawk's falls to 0, skills.md § Hawk's
    Stealth), within its field of view (its placement's, else its type's `half-fov`: 60 or 90
    degrees either way of its facing, or of its aim while it fires), with nothing of the world or
    the pillars between its head node and theirs (the pair table's ray, #244: [brain.md](../brain/sight-and-contacts.md)
    "The sight ray"). A contact unseen 15 s,
    stealthed to 0.2 or gone is forgotten. One who hits it becomes a contact at any range, and
    with nothing to fire at it turns to the last place of a contact seen in the last 3.75 s, so
    one shot from behind turns and fires back. It fires at the nearest contact it sees within its
    weapon's range (at least 20 m; the shot's own limit, not sight's). `BF_SIGHT_LOG=1` prints
    each one finding or losing its target and its contacts, and the muzzle corridor to each
    contact when it changes. It faces them, turns the gun onto
    them and fires in its bursts (the existing random delay, bursts and pauses); the aim is
    exact. It fires only with a clear muzzle corridor to the target (#244, [brain.md](../brain/sight-and-contacts.md)
    "The muzzle corridor": a body in the line, a dummy or a teammate, holds it, with the gun up).
    Its shots stop at the first body on their line but its own, and hurt by the held weapon's
    damage when they get there (`AiHits`). In the main
    game everyone is on team 0, so nobody has a target and nothing changes. The target
    (`Player::ai_target`) is the AI's only: it's dropped when control passes to a character
    (the number keys, the hand-over after a death) and never kept by the controlled one, whose
    gun and body follow the crosshair from the first frame.
  * A **follower** (#364, the product owner's request, not the game's: `Npc::follow`; the
    apostrophe on the NPC under the crosshair, or in the menu for the next one; `BF_TEST_NPC`'s
    `+follow`, `BF_TEST_NPC_FOLLOW`) trails the controlled character by the squad's own follow, in
    FOLLOW (Cover Me): play.rs `update_player` hands it to `orders::think` as it hands a squad
    member, so it runs play_follow.rs's states ([squad.md](../squad/the-follow.md) "The follow"): it holds,
    kneeling and looking about, while he's within 11.2 m; beyond that it chases a route to his
    point at the dash (over the level's world points where the level has them, straight on the
    test map's floor), stops within 9.8 m of him once he isn't running, and takes a cover point
    near him where the level has one (the filter looks through the followers too, so two never
    take one point, nor one a squad member holds). Its routes, steering and speeds are the
    squad's (play_routes.rs, play_avoid.rs, play_speed.rs; a type of its own, a marine or a
    Shrike, moves by its clips' root as the tutorial's marines do, [brain.md](../brain/confed-marines.md)). It's no
    squad slot: it joins no formation list, so its leader is always the controlled character,
    whoever that is after a hand-over, and no order, hand-over re-order or target broadcast
    reaches it. It combines with the two behaviours above: set to fight it fires at the hostile
    it sees as it trails him (the upper body on the target), a dummy never fires (and stays the
    player's target, nobody else's). Set to stay again it stops where it is, facing the way it
    was going, its cover point given up. Only a friend of the controlled character (the same
    team) follows: an enemy keeps the setting (its message says "idle: only a friend follows")
    and stands; so does every follower while the map editor's stand-in leads (edit mode), until
    play starts again. The squad's members, the enemies and the level's characters run what they
    ran.
  * NPCs take no pickups (medkits and fruit are the squad's).
  * Removing one takes its body; the gun it dropped, its blood and its DNA stay.
  * Any loaded character type can be spawned (#128). The test map's own levels define 55
    (common's squad, e01's, mp_common's deathmatch and squad deathmatch types, m02_a's and
    m09_a's); it adds the other 73 of the data's 128 from the level archives that define them
    (#180, `BF_TEST_TYPES`, below; character-types.md, "The test map's other types"): the
    terrorists (Red Hand, Terrorist, Subversive), Kingman, Gunthar Ghent, McTavish, the
    Militia, the Supermutants, the ferals, the Seers and the Shadoon, the Hunter Lord, the Fire
    Hounds, e34's, e40's and m14's Shrikes and the campaign's other copies of the marines,
    mutants and Brutus. The menu names each by the archive it came from (`Red Hand (e05, 31
    hp)`), as `name@archive` does. A squad type is that squad slot's character; any other is slot
    `OTHER`, with its own model, clips, face, HP, weapons, flesh and blood decals, and it takes
    damage by its type's factors, hit parts and damage-multiplier (play_damage.rs). It runs its
    type's own behaviour (#129, [enemies.md](../enemies.md)): the MOTION table's clips and the
    type's speeds, hit reactions by its personality, the game's ragdoll (every character's,
    [ragdoll.md](../ragdoll.md)), its death line from its type's chatter, and on Shrike flesh green
    blood and the game's decal rule. It says nothing else and has no skill; its ground is the
    squad's (the sole lift); its
    capsule is the squad's and its skin's glow isn't drawn (#173). The other species' tickets
    (#130-#134) check theirs. See character-types.md, "In play" and "Not done".
  * A **frozen** one (`BF_TEST_NPC`'s behaviour) is a dummy with its brain off, as the takes'
    trainer freeze: no hit reactions (enemies.md).
  * Several characters can share a squad character (an NPC Tex beside the squad's Tex), so
    what reaches a body later tells them apart by `Player::who` (a squad member's character, an
    NPC's number): Energy bolts and Sonic rings land on the body they were drawn to, and the
    thrower's own share of a Frag, Gas, Energy or Sonic blast is the squad thrower's only, not an
    NPC's of the same character.
