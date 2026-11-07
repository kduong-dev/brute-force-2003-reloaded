*Part of [testtools/](README.md), the former page `testtools.md`.*

* **Scenery** (`play_testscenery.rs`, #185, the first part of the map creation epic #190): any
  level's buildings and static scenery, on the test map or on any level the session is on.
  * Which pieces (`src/bf/pieces.rs`): a level file's `<game-object>` whose object type's
    `<base object-type>` is 2, OT_STATIC_GEOMETRY by the game's OT_ name table (0x3be87c), with
    a mesh archetype. The game fixes such an object's physics bodies and never adds it to the
    world's list of objects that move (FUN_001590a0: types 2, 26 OT_PLATFORM, 31
    OT_WORLDBUTTON). Left out: the types that break (a debris list, h_197caf14: the object
    menu's kind, FUN_00157260) and the doors and gates (a part on a sliding joint, h_fd8f670c:
    they need the door machinery). Every other model a level places has its own element
    (characters, pickups, weapons, liquids, world buttons, platforms, turrets); its other 77
    game-objects are OT_BODY (object-type 0, loose bodies). In the 53 level files: 23,670
    game-objects, 19,671 of them static scenery (3,823 break, 99 are doors), of 675 types; 2
    (mp3) to 80 (m03_b) types a level, 2,412 entries counted level by level.
  * The list: the levels in the level menu's order (the squad deathmatch maps, the deathmatch
    arenas, the missions' zones), each its pieces, the largest first: its type's name (the data
    names none, so the hash), its size across (twice its bounding radius, `h_1ab09530`, about its
    origin), its parts, "no collision" for the few without a collision surface (grass, cables,
    backdrops), and how many its level places ("x11 in its level"). Read when the menu first
    opens, once a session: each level archive up to its objecttypes (its level file, objects and
    objecttypes; not its models, textures or sounds), on 8 threads, with common's types and
    archetypes first as the game looks them up (FUN_00160da0, the level file's libraries
    "common <level>"). `BF_TOOLS_LOG=1` prints every entry ("sdm_e34:0 h_0baf091a 58 m, 19 parts
    (x1 in its level) arch h_0baf091a").
  * Building a piece: from its own archive, the only data it needs (all 53 checked: its objects,
    materials, textures and collision surfaces; two of common's types placed in five missions
    are built from common.tgz, and 16 texture references in m01_a, m05_a and m05_b are common's).
    A picked piece's archive is read on a thread into a Game of its own (`Game::load_scenery`:
    its objects files whole with their .ivd / .ipn, its materials and its textures, with the
    test map's textures shared), and the piece made from it as its level makes its objects
    (`level_scene::object_parts`, the code `spawn_level` uses: cut-outs, glows, grass lit from
    above, glow decals), lit by the map's lights (on a level, its lamps too), casting no shadow.
  * A piece's origin goes where it's put, as its level places it: on the ground, a building's
    floor is the ground's (what its level sinks into its terrain goes under the floor). The
    wheel turns it about its origin.
  * Collision: its parts' collision surfaces (`arena::physics_tris`) are added to the map's
    arena, solid like its own placed objects (`Arena::place`: the arena is copied with them
    added, as it's shared read-only; 7-12 ms on m09_b's 149,166 triangles, none for a piece
    without collision). The squad stands on it and is pushed out of its walls; shots,
    grenades, missiles, the crosshair and the AI's sight meet it. One with no collision surface
    is walked and shot through, as in its level.
  * The flat floor has no arena: the first piece with collision put on it gives it one
    (`Arena::flat`) of its ground, a plane at the floor's height under every point (terrain,
    material 0). Its pillars
    and the boxes of the breakables the object menu put down stay its own (`ray_hit`,
    `hit_normal`, `pillars` keep them on that arena). Before a piece is placed nothing changes;
    after, the floor's code paths are the arena's, which on flat ground give the same (the NPC
    run below: the same combat log, the same frames). What does change: the follow camera stays
    out of the pieces and above the ground, as on a level.
  * Memory: one archive's data is held at a time (the one a piece was last built from, 4-16
    MB: dropped when another's is loaded), and the pieces built: every placed one, and at most 6
    others (`KEEP_UNPLACED`), the least recently picked dropped first with their meshes and
    materials. A map switch drops them all (the menu's state is a map's); the list is the
    session's.
  * Hooks wait for what they need: a pick or a placement made before the list is in waits for
    it, and a placement for its piece to be built (placed the frame it is).
