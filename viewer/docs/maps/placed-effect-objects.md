*Part of [maps/](README.md), the former page `maps.md`.*

* **Placed effect objects (#258)** (`ale_fx.rs` `spawn_placed_effects`, `play_placed_fx.rs`;
  `bf/level.rs` `EffectObject`). The air particles a level places: the tutorial's 18 swarms of
  flies (xfly), 7 of butterflies (xbfly) and the lava yard's 3 clouds of embers (slava-fuzz),
  and other levels' fires, torches, sconces, steam, dust storms and power-up spot glows (1453
  placements over the levels; m01_a has 65).
  * **What the game does.** An `<effect-object>` (object-type 17) is an object of an effect type
    (objecttypes `<effect>`). The world ticks it from the level's start (0x21b7a0), and it made
    its ALE effects as it was set up from its type (0x21a180, FUN_00219fc0): it runs its effect
    type's ALE effects and its light effect where it's placed. A spawn-trigger's template isn't
    in the world (FUN_000dfe90): only its copies run (the tutorial's lava jet and farcast pads,
    "Effects the triggers make"). Nothing else starts these: no trigger names the tutorial's
    flies, and their `lifetime` is FLT_MAX.
  * **The object.** Its life is the placement's `lifetime`, else its type's `max-life`: a 0 is
    taken away at the first tick (e01's exp_gasmain, never seen); every other placed life is 90
    days or more. Its type's h_0c88c46c is the user parameter it starts with (FUN_0021c380, 1
    on nearly every type), its type's bounding sphere its distance LOD. Each emitter runs once,
    for its lifespan (0x2a5b10): the ambient ones for 1e10 s, the few explosions levels place go
    off once at the start. An alchemy entry with a creation-time starts that much later (m14_c's
    meteor strike, 2 s).
  * **Camera-attached types** (h_1e40fd68, the type's flag: the placement's own isn't read,
    0x21c000) are kept at the camera each frame, h_0e528ccb m along its view (FUN_0021a700). The
    levels place eight, all at 0 m: six start with their level (e01's dust storm, m03_a/b/c's,
    m14_a/b's meteor rain), and m09_b/c's two are spawn-trigger templates the demo doesn't
    spawn yet. Their particles stay where they're born.
  * **Taken away** by a destroyer-trigger that names one (play_triggers.rs takes them in with
    its copies: e34's six tutorial pointers, e40's io tablet glow ...), or with the breakable
    object it's attached to (`<parent Type="1">`, FUN_0021a700: e01's barrel fires go when
    their barrels have gone, on the frame the model does). Either way the effect object is
    destroyed, and every particle and light it had made goes with it on that frame (#308:
    `ale_fx::TakenAway`; the game's destructor takes every node of its ALE effects out of the
    manager, FUN_0021a4d0 -> FUN_0021c700 with AL 0, so none lives out). A hardpoint link
    (Type 2: the torches and sconces) doesn't take it away (FUN_0021a700 acts on a type 1 link
    only), but the holder's removal stops the effects it lists (FUN_0015a1d0): the emitters go and
    the particles out finish, so the flame dies down and its light fades. What the game does, and
    how it was checked: [scenery.md](../scenery.md) "What hangs on an object when it goes", #308.
    An object link that finds no object at all (`<parent Type="1" parent="">`: e16's
    h_1f4f877f) is taken away at its first tick, so `spawn_placed_effects` never starts it (#327).
  * **Drawn** with the game's particle rules (the triggers' effects' shapes, count once a game
    frame, facing and LOD; the shapes are the table at 0x402a4c, where a shape-2 quad spans -1 to
    +1 of its Scale, twice the demo's own quad, and a shape-3 one stands on its particle) and
    more read from the game's code for these (`Compiled::placed`):
    * every particle starts in its emitter's box, which reaches its Width / Height / Depth to
      either side (both cube classes' spawns draw [-1, 1) of each: 0x2b17c0, 0x64e30; a game
      cube's OffsetHeight h_e2999ffd from the emitter up), and none is attached; FxCubeEmitter's
      polar spread is MinSpread + r x (MaxSpread - MinSpread) with r in [-1, 1). So xfly's flies
      start anywhere in 39 x 18 x 39 m, centred 12 m above their object;
    * a sphere emitter's particles start MinRadius + r x (MaxRadius - MinRadius) out along their
      direction (0x2a8a90) and a cone's the same out along their own way, its polar angle
      MinSpread + r x (MaxSpread - MinSpread) (0x2aa190), each r in [-1, 1). So a smoke
      column's puffs (0.04-0.21 m, spread up to 30 degrees) start within 21 cm of their emitter
      along their rise, where the demo's own reading (the other effects') puts them out flat
      from the cone's axis;
    * every facing-3 appearance is drawn stretched along its motion and turned about it to face
      the camera (`Facing`): the draw picks the shader's facing by GeneralApp_FacingType
      alone (0x2a1de6), as every effect does since #286 (**Particle facing**). So
      e01's barrel and trash fires, its campfires and m01_a's braziers are flame tongues rising
      from their coals (spun at random about their foot they were a flat star of flame), and
      the godrays (e06, m01_a/b: 4 m quads up to 70 times as tall, falling at 0.06 m/s from 171
      m up) hang as pale shafts;
    * turbulence fields run (FxTurbulenceField, 0x2a9440 / 0x2a9590): once a game frame each
      particle's velocity is pulled by dt x Approach x 4 of the way toward a new random
      velocity, a random direction x r x Magnitude with r in [-1, 1). xfly's 96.8 and 0.73
      keep the flies darting about at ~13 m/s (the root mean square in the demo), turning
      within a few tenths of a second; the butterflies' 24-35 and 0.20 at ~1-2 m/s.

    The other effects (the grenades', the guns', the triggers') keep their own checked rules:
    the box and the spawns for all of them are #164, the fields #101.
  * **Held out:** the five placed effects whose curves vary with the user parameter
    (s-lavalight, s-lavapool: the lava pools' glows; slava-cavern-splash, duststorm_med,
    env_fl_mdsplash). The game draws them at the parameter their objects start with, 1:
    FUN_0021c380 sets it before FUN_00219fc0 makes the slots, FUN_0008f5d0 hands it to each ALE
    effect (the effect's +0x18, 0x2abaa0), which sets it on each node (0x66a00, the +0x88 its
    curves are read at). There s-lavalight is 14 a second 4.9 m sheets of sidedistortion2 up
    to 21 m tall over a 23 x 22 m box, added at an alpha of 0.108: a faint sheet. The demo draws
    it far too strongly, an orange curtain over the whole view of the tutorial's lava yard that
    take08 doesn't show (100-134 s: the far hills stay clear behind it); the likely cause is the
    faint added sheet taken as linear light (`Source`, #165's blend). They stay out until that's
    drawn as the console does (`BF_PLACED_FX_VARIANTS=1` runs them anyway).
  * **Budget:** the placed effects' particles have their own cap (4000; m01_a keeps ~2000
    alive, the tutorial ~300, e40 ~600, sdm_e34 ~430), so they never crowd out a fight's
    (3000). Both are counted afresh each frame from the particles there: a count kept from frame
    to frame missed a map's end, which takes them all at once, so m01_a loaded a second time
    drew half its torches and a third time none. Frame time (bf_play, 40 s, mean): the tutorial
    5.5 ms against 5.2 without them, m01_a 5.6 against 5.1, e40 5.3 against 5.1.
  * **Not done:** their sounds (the fires' and torches' loops) and damage areas (e34's exp-oxy),
    a finite life's end and the start over of a type flagged h_0d5e990b (FUN_0021c500: none
    is placed with a life under 90 days), each alchemy entry's own max-life (the slot's +8,
    which FUN_0021a850 reads: FLT_MAX on nearly every entry, 10 s on h_f90c728a's in m01_a/b
    and m05_b/y, 207 and 600 s on a few in e01, m01_a and sdm_e01; the demo runs them on), the
    lights levels hang on them (`<parent>` light objects of type h_09020c30: #143's; the game
    drives one such light-object from the light effect, FUN_0021d270, and the demo draws that
    light effect's one lamp, the newest particle's, since #272: levels.md "Linked lights",
    "Effect lights").
  * **Over the sky (#290).** Until #290 the sky's see-through layers were drawn after every
    see-through thing in front of the camera whenever the camera was level or looked down, and
    painted over it wherever the sky was behind it ([levels.md](../levels/sky.md), "The sky is drawn
    first"). So the placed effects showed against the ground and stopped at the skyline:
    sdm_e13's embers (slava-fuzz) showed only on the lava and the rocks, none in the air above
    them. It wasn't their placement: every slava-fuzz particle is born 1.6 to 36.5 m above its
    object (`BF_ALE_DUMP=slava-fuzz`: 256 births in a 10 s run, mean 18.6 m, drifting down at
    up to 1.23 m/s), the box the game's cube spawn makes (0x64e30, through the emitter's -180
    degree frame by FUN_000621c0), and `BF_ALE_LOG` has all eight objects emitting, with 787
    particles alive over all the level's effects at 7.5 s, far under the 4000 budget. Every
    effect, placed or not, whatever its facing, shared the fault, and every level with a sky
    (51) has a see-through layer. Main and the fix give the same picture where nothing
    see-through lies over the sky (fixed-step captures, same seeds), so their difference shows
    what the fix brings back (scratchpad t290dev, frame 75, 5 s into the capture; pixels
    changed by 8 levels or more):
    * sdm_e13 at the south shore, level (`BF_MAP=sdm_e13 BF_TEST_GOTO=-19.97,-122.8,-19.97,-122.8
      BF_CAMERA_PITCH=0`): 4.0% of the picture, the embers over the rock walls of the sky; looking
      down with Brutus (`BF_CHARACTER=0 BF_TEST_GOTO=-19.97,-122.8,-19.97,-140
      BF_TEST_GO_AT=999`): 1.7%, the corners where the sky shows; pitched up 0.35: 0.06%, a
      pixel's fringe along the skyline (the terrain's blended layers, 0.02 m above the ground,
      now drawn over the sky's edge, as the game draws the sky first).
    * The tutorial's lava yard, level (`BF_MAP=tutorial BF_TEST_GOTO=50.68,17.53,55.8,8.9
      BF_TEST_GO_AT=999 BF_CAMERA_PITCH=0`): the sky was empty; now the embers fill its left
      half with three flies (xfly) among them (0.47% of the picture changed, growing from 0 as
      the embers build up), as take08 shows its sky (Checked, below; #258's check saw them
      because it was pitched up 0.25).
    * m03_a's dust storm (ferg_duststorm_light, camera-attached: `BF_MAP=m03_a
      BF_TEST_GOTO=-221.61,-113.87,-188.2,-51.8 BF_TEST_GO_AT=999 BF_CAMERA_PITCH=0`): 5.4%,
      growing from 0.04% at the capture's start as the storm builds up: a faint veil of dust over
      the sky (its top left 4-8 levels darker) and the odd speck of debris, which main didn't
      draw.
    * m14_a's meteor rain (smeteor_rain_master, camera-attached: `BF_MAP=m14_a
      BF_TEST_GOTO=-104.3,-163.1,-104.3,-150 BF_TEST_GO_AT=999 BF_CAMERA_PITCH=0`): no change in
      any of the six frames kept (0 to 5 s into the capture); nothing of it crossed the sky in
      view there, so it isn't checked.
    * The tutorial's landing spot, looking down (the recipe below): a band along the horizon
      changes from the first frame, the sky's own layers in their order now (levels.md): a pale
      haze over the far island where main had a dark band and ghostly pale palms, as take03 at
      31.5 s shows it (a blue-grey haze, the far palms muted).
    * Not checked against a take with the camera level or up on sdm_e13: the takes there all
      look down at the lava (#146); a take pitched up at the rock walls is A1 of
      todo/286-particle-facing/shotlist.md, not recorded. #146 take01's frames at 1 and 4 s
      differenced show the game's embers all over both players' views, near ground and far
      shore alike, and #28 take08 shows the tutorial's in its sky.
  * **Checked** (scratchpad t258dev):
    * The tutorial's landing spot (take03 30-33 s: `BF_MAP=tutorial
      BF_TEST_GOTO=65.55,74.03,55.6,73.05 BF_TEST_GO_AT=999`): dark winged flies dart about the
      sky over the hill and the palms, one to a few in view at once, as in take03.
    * The lava yard (take08 45 s: `BF_TEST_GOTO=50.68,17.53,55.8,8.9 BF_TEST_GO_AT=999
      BF_CAMERA_PITCH=0.25`): the sky above the pits full of drifting light specks, the embers,
      with flies and butterflies; take08's are as many by a dot count through the takes'
      encoding (the demo's frames scaled to 480 lines, x264 crf 16: ~80 specks of contrast 25
      or more in the same region, take08 ~80), but the demo's are warmer and crisper.
    * The pool at 120 s (`BF_TEST_GOTO=46.26,-25.92,51.4,-34.5 BF_CAMERA_PITCH=0.12`): no
      curtain, as take08 (the orange light on the hill there is #143's).
    * e34: `BF_TRIGGER_FIRE=h_f211fe52,6,4.0` removes the tutorial pointer by (69.2, -92.5)
      ("removes the effect h_f3bee12f"). e01: `BF_TEST_HIT=5.0 BF_TEST_HIT_NEAR=-110.96,51.77`
      breaks the fire barrel, and its fire goes with it. Since #308 the particles and lights
      go on the same frame (scenery.md, "Verified": before, the pointer's glow faded over 0.9 s
      after its removal and a barrel's lights for 1.3 s after its break).
    * The tutorial's opening: the lava bursts and the farcast come once each, as before (the
      templates are left to the triggers); e40, sdm_e34 and the test map play as before.
    * m01_a loaded three times in one test session (`--test` with
      `BF_TEST_LEVEL=1,m01_a;30,m01_a;30,m01_a BF_ALE_LOG=1`): each load starts from 0
      particles and keeps ~1950 alive, the same each time; so does a map played after
      `BF_TEST_BACK` has gone back to the menu.
    * e01's fire barrel (`BF_MAP=e01 BF_TEST_GOTO=-110.96,64,-114,50`, frames 100-140): flame
      tongues stand up out of the coals and over the rim. Until #272 the bright flat disc over
      the barrel's mouth was its light (light_barrelfire: several point lights 0.3 m up inside
      the barrel, 4.7-7.3 m reach, 1 / d^2); now one lamp, (0.69, 0.60, 0.26) reaching ~4.8 m
      with the game's falloff, and the coals show as glowing embers (levels.md "Effect
      lights": the mouth's patch 238 238 182 and 79% saturated before, 122 91 52 and none now).
    * m01_a (`BF_TEST_GOTO=30.6,29,30.6,32`, frame 89): the godrays hang as pale shafts in the
      mist beyond the trees.
  * **Doesn't match yet:**
    * The flies look about 1.65 times as big as take03's and take08's and cross the picture
      about twice as fast; their speed for their size on screen matches within 25%, so the
      turbulence (read from the code: ~12.6 m/s root mean square) is likely right and their size
      is off. 1.27 of it was the demo's narrower view, the game's since #269
      ([camera.md](../camera.md): about 1.30 should be left, not measured again); the rest wants a shot of a fly at a
      known distance in xemu. Their size is xfly-basic's Scale x 2 (shape 2, 0x402a4c).
    * The embers are warmer and crisper (the demo's additive blend in linear light at 720
      lines, #165), and they come and go in waves of 5-10 s (1 to 42 specks within 6 s).
      Nothing in slava-fuzz is periodic: its rate (4.68 a second), life (7.97 s), speed and
      transform are constants, and each particle fades in over its first quarter and out over
      its last fifth. It keeps ~37 alive, each a 14-35 m sheet of noise2's dots; the waves are
      the few sheets in front of the camera coming and going. Not checked against a take's
      steadiness.
    * (e01's fire barrels' light burnt the coals out to a flat yellow disc: the lamp rule is
      traced and drawn since #272, levels.md "Point lights". What's left is the arithmetic of
      lamps over lit surfaces, "The arithmetic" there, and the ground, unlit, taking no lamp.)
