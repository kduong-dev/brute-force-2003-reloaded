*Part of [grenades/](README.md), the former page `grenades.md`.*

## Thermo (#82)

"Powerful timed explosive charge. Detonates 15 seconds after armed." Defined in m09_b alone
(h_0141ea85, its label h_e9c9f9d2 "Thermo"): `timer` 15, group-type 3 (IG_EXPLOSIVE) item 2 (it
comes between the Sentry (3, 1) and the Roller (3, 4) in the store's order), h_1ee2f4ed 2
(IOU_PLACE_ON_GROUND), function-type 0 (no item set of its own), stack-limit 5, mass 23, its HUD
icon h_0ebe1a32; its explosion h_08f5cb41: Damage 19.5-97.5, damage-type 10 (EXPLOSION),
radius 3, core true. The level places it once (h_e04e05e1 at (-178.4, 58.3, 126.5),
levels-m09_b.xml 938), and not as a pickup: it's set down live (h_1cb7d5f2) on the enemies' side
(item-alignment 1), as the tutorial's mines and e16's, e34's and e40's Sentries are. The item
object's handler (0x14a0a0) lets it go as a throw does (+0x1bc bit 1, its item set's +0x10,
FUN_0014a150's steps). Nothing in the level refers to it (no trigger, script or objective): a
prop that goes off on its own. The leads and the takes are `todo/82-thermo-grenade/`.

* Nothing of its own to build: it goes the placed types' way (**Placing** above). G sets it down
  at the feet with the stance's place_hi clip (the count drops at the press), its fuse runs
  from when it's down (the grenade's contact handler 0x149510 starts it, as every type's), and
  15 s later it goes off with its effect type's ALE effects, light and sounds, its decal, and
  the shared blast's damage (#112). Function-type 0: the base item set (0x39bec0), whose slots
  are stubs; no detonator (IFSET_DETONATOR, 17, is on no item).
* **The level's charge** (`place_mines`, play_sentry.rs): set down where it's placed as the level
  loads, team 1, owned by nobody, its fuse armed (its body settling on the floor is a first
  contact: in take01 it lies 2-3 cm off its placement), counting while no cutscene holds the world
  (**The fuse**). m09_b's startup cinematic (m09_b_startup.gscr, 12.035 s) sets the cutscene bit
  with its 122 at 0 s and clears it with its 123 at 9.0 s, so the charge goes off **24.0 s after
  the level's start**, wherever the squad is (take01: held at 15.000 for 6.3 s of its snapshot's
  clock, which was taken ~2.7 s into the cinematic, then 15.0 s to the blast at 21.3 s). With
  `BF_TEST_GOTO` or `BF_SCRIPT` (no start scripts) it goes off 15.0 s after the load. Going off it
  sends DESTROYED as the level's mines do (nothing listens). The rule is every live grenade item
  the level sets down, not the Thermo's: the Sentries as before, but not a Roller (m09_a's one is
  left out pending #316, see **Roller**); a spawn-trigger's template stays out of the world (none
  of the Sentries is one).
* **Its blast**: the effect type h_fbdd1258 has the Roller's h_013b3ca8's content (the Frag's six
  ALE effects, light_explosion, h_065168c9), at 3 m, plus the impact sound h_16611581; no
  per-material ground puff (its explosion's h_0cbcb8f2 is empty, where the Roller's has dirt and
  stone entries; the demo draws none for any grenade yet).
* Blocked (not filmed, not traced): whether a shot sets it off (take03's round landed ~0.3 m from
  it and it didn't go; its 1 hp and the item's damage path aren't read), so it isn't among the
  shots' targets as the Sentries are; its damage to enemies by distance and to squadmates (only
  the thrower at 1.4 m is measured, take04: Brutus -13.9, the core blast's 0.42 of a roll by the
  code, #112's); its look (the mesh h_0aac8518 is never seen lit in the takes) and sounds
  (h_1d85a923's reader isn't traced); the AI's reaction to it.
* Its kit is made where the level defines it: `load_kits` takes in the grenade and explosive types
  a level puts down itself (its item spawns, h_1d1d4f05, as pickups or set down live), so m09_b
  loads the Thermo (`8 grenade types: Frag, Gas, Sonic, Energy, Light, Sentry, Thermo, Roller`).
  The rule is general, not the Thermo's. Run on each level that puts down a type outside the
  squad's seven (28: e16, e23, e34, e40, m05_c, m07_c, m08_a, m08_b, m09_a-c, m09_x, m14_b, m14_c
  and the fourteen arenas; `BF_MAP=<level>`, the kit list it prints), it adds a kit on four:
  m09_b's Thermo, and the Sentry h_f73de83d on e16, e34 and e40 (6, 6 and 5 types, one more each
  than before). On the others what they put down is loaded already under its label (another
  definition of the Sentry or the Roller) or isn't a usable item. Those three levels lay that
  Sentry as live mines, set down with h_1cb7d5f2: e16 14, e34 6 and e40 7, item-alignment 1 (the
  enemies') but one of e34's, -1. With its kit made, `place_mines` (play_sentry.rs, #228) now arms
  them as the level loads, as the game's handler does (0x14a0a0): `BF_SENTRY_LOG=1` on e34 prints
  six "the level's mine ... set down armed" at t 0.00, h_0a8cb601 on team 0 and five on team 1, the
  enemies', which go off for Brutus, Flint or Hawk within their 3 m and for Tex only stepping on
  one, as the tutorial's do. Before #82 they had no kit there and weren't placed. Not checked
  against footage of those levels.
* **A live mine of a type that shares the squad's label has its own kit (#317).** `load_kits`
  took a level's own types by label, so a Sentry defined under another hash than the squad's
  h_e5f1f063 got no kit wherever the squad's own was loaded, and the level's live mines of it
  were never put down. It now also loads the type of each live-placed item (`Placement::placed`,
  h_1cb7d5f2) by its own hash (not a spawn-trigger's template; a Roller stays #316's), after
  the label rule, so the squad's kit keeps its place first and `GrenadeKits::find("Sentry")`
  still finds it. m09_b's live Sentry h_f73de83d is set down (`9 grenade types: Frag, Gas,
  Sonic, Energy, Light, Sentry, Thermo, Sentry, Roller`, the second Sentry sorted by its
  (category, index) after the Thermo), m09_x's four live Sentries (`8 grenade types`: one of the
  squad's type, which was set down before, and three of h_f73de83d) and e23's three, of h_ecfe0d8c (`6 grenade types: Frag, Gas, Energy,
  Sentry, Roller, Sentry`; function-type 8, explosion h_01f142eb radius 4 as the squad's). Each
  mine is its own kit's: `place_mines` takes the kit by the object's type hash, so its radius
  and blast are its definition's. `BF_SENTRY_LOG=1` on every map before and after (53 runs,
  12 frames each): the placed lists are the same on all the others (e05 4, e10 6, e16 14,
  e34 6, e40 7, m03_a 5, m03_b 5, m03_c 4, m08_b 14, m14_a 7, m14_b 1, m14_c 2, the tutorial 3,
  ...), only e23 (0 to 3), m09_b (1 to 2) and m09_x (1 to 4) change. The squad's start
  stacks are credited by label (`StartStore::grenades`), so with two kits of one label the
  first, the squad's, takes them and the others start with none (else the stack would be
  carried twice): `BF_INVENTORY_LOG=1` on e23, m09_b, m09_x and e34 shows no Sentry stack
  (none of those zones carries one), and m14_c's `Sentry 10` is as before. Not checked: the new
  mines going off. Tex can't walk to them from the start (held at z -93.5 short of e23's
  first, (-67.3, 25.4, -97.9), and at z 78.3 short of m09_b's, (-125.5, 57.5, 86.9)), and
  `BF_TEST_HOSTILE=hawk,4,0,1` didn't set e23's first off (no check line at all, not looked
  into).
* Blocked: how the squad comes by one. The data has no Thermo pickup (its one placement is
  the set-down charge above): pickups are #17's, disarming a set-down item #145. Also blocked:
  what m09_b's mission does with the charge (its objectives and triggers, the campaign). Until
  then the squad has none but by the test hooks.

Checked (`BF_GRENADE_LOG=1 BF_COMBAT_LOG=1`): on m09_b (`-- --test BF_MAP=m09_b BF_CHARACTER=3
BF_TEST_INV=Thermo:3 BF_TEST_GRENADE_TYPE=Thermo BF_TEST_THROW=2.0,0.1`; m09_b starts in a
letterbox that hides the HUD: `BF_TRIGGER_FIRE=trigger_letterbox468,USER_SIGNAL1,0.05` lifts it,
the tester's) the press at 2.0 s drops the count 3 -> 2, the clip lets it go at 2.53 s, it's down
0.6 m from Tex at 2.93 s and goes off at 18.00 s (15.07 s, a 15 fps step), Tex 1.1 m from it taking
his share (14.7). On the test map with m09_b's data
(`BF_TESTMAP_LEVELS=mp_common,mp1,mp2,mp3,mp4,mp6,mp7,mp8,m02_a,m09_a,sdm_e34,m09_b`, the same
hooks, `BF_TEST_GOTO=0,30,0,60 BF_TEST_GO_AT=100 BF_TEST_INSTANT_KILL=0`, dm Shrike Soldiers frozen
1.8, 3.5 and 3.9 m from where it lies): the one at 1.8 m takes 97.5 x (1 - 1.83 / 3) x 1.4 = 53.2
(its stamina shield 25 of it), the two beyond 3 m nothing. The item box shows its icon, "Thermo"
and the count.

Checked (#82, `BF_GRENADE_LOG=1 BF_SENTRY_LOG=1 BF_SCRIPT_LOG=1`): `BF_MAP=m09_b` from the level's
start (no `BF_TEST_GOTO`): "the level's grenade h_e04e05e1 (Thermo) set down armed at [-178.37,
58.25, 126.47], team 1" at t 0.00 (take01's object at (-178.386, 58.273, 126.433)); m09_b_startup's 122 at
t 0.00, its 123 at 8.97 (script 9.000), and the blast at **23.97** (9.0 + 15.0), with 6 effects,
light 0f220d41, h_065168c9 + 16611581 and the decal f5ccedb0. From take02's spot
(`BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_GOTO=-184.22,117.80,-108.7,183.4 BF_TEST_GO_AT=100
BF_CAPTURE_FPS=15`, Brutus 10 m from it facing it): the blast at 15.07 (15 s after the load, no
start scripts), Brutus untouched; frames 220-249 beside take02's 10.7-12.6 s: the same doorway, a
white core at the floor line on the first frame (the take's 11.0-11.15), the yellow-white
fireball filling the doorway at +0.13-0.4 s (11.35-11.5), orange fire and brown smoke at +0.6 s
(11.7), an orange haze after; the demo's haze is dimmer and gone sooner than the take's (>2 s),
the shared effects' rendering.
