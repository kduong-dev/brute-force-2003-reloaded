*Part of [play/](README.md), the former page `play.md`.*

### Radar (#228)

`play_hud.rs` (`update_squad_hud`, `update_radar_marks`) and `play_radar_disc.rs` (the disc, the
circles and the wedge), from the radar widget's code (FUN_000ae580 and the functions below;
[code-map.md](../code-map/hud.md), "HUD"):

* **Its place** (#347): the radar panel's corner is (56, 301) (`RADAR_FRAME_AT`; FUN_000c2be0:
  the HUD safe area's left + 5 and its bottom - 10 less the panel's 128), the radar's middle
  (120, 365) (`RADAR_CENTRE`), the disc, the frame, the dots' overlay, the tabs and the brackets
  all from it. The demo had them at (57.5, 305), 1.5 right and 4 down (#333 found it). The takes
  agree: the frame texture's edges slid over take03 (1.0 s) and take07 (5.0 s) of the tutorial
  fit best at (56.0, 301.0) of 2337 corners 0.25 apart (the old corner ranked 1926th and 1675th;
  the demo's frame, searched the same way: (57.5, 304.5) before, (56.25, 301.0) now), and take07's
  viewer dot sits at (119.6, 364.9). The faces round it are placed by the game's table too (see
  "The portraits").
* **The disc, the circles and the wedge** (#347, `play_radar_disc.rs`; the radar widget's draw
  0xaefb0, FUN_000af1a0; [code-map.md](../code-map/hud.md)): three passes over the 128 x 128 quad at the
  corner, as the game's textures and pixel shader make them, not the stretched disc and
  procedural cone they were:
  1. **The disc**: h_17d0ee37 (a teal ball, alpha 0.53, 0.6 round its rim; the circle in it
     86 units across, as take03's), white, at its own size.
  2. **The squad's circles**: the living members in the viewer's squad (and in reach, as their
     blips), up to three, each the ring texture h_1fbc4e99 (a hard-edged disc) round its point at
     the radar's scale, the radius its type's `radar-range` times 0.325 (60 m: 19.5 units, the
     disc in the texture 1.05 x that), their alphas added and clamped at 1, times the disc's own
     alpha; coloured (disc x (0.6, 0.4, 1.0) + (0.4, 0.2, 0.2)): lilac over the teal. The game's
     pixel shader 0x24 does it (its D3DPIXELSHADERDEF at 0x3e0320, its combiners and the
     constants from 0xaf524 decoded in `play_radar_disc.rs`).
  3. **The viewer's pass**: its own circle (a dead viewer has no record: none; a living one whose
     type has no `radar-range` has one of no radius, FUN_000aef00 refuses only the dead: its wedge
     is drawn without a circle) and the wedge,
     h_fcebfc70, a triangle with its apex at the middle, stretched across the quad by f = 0.5 /
     tan(half the camera's field of view across): its texture 52.7 units wide at the quad's top
     for the follow camera's 70 degrees (the triangle's base is 114 of the texture's 128 texels:
     about 46.9 units), a hairline at the L-Shot-75's 25x (take06 of `todo/62-l-shot-75`
     at 6.0 s shows the line against 1.0 s's wedge; the demo's `BF_TEST_SCOPE=2,1.0` the same),
     coloured disc x 0.4 + 0.3, pale grey, the same as the viewer's circle. A lone viewer
     (the tutorial's Tex, take07) shows one shape, a squad's circles lie under it (take03).
  The colours, as the game blends them (stored values, SRC_ALPHA / INV_SRC_ALPHA): over take03's
  ground the six zones of the disc, the circles and the middle measured in the take come out of
  the formulas to within 4 levels (mean 1.6; e.g. the disc at (-30, +25) from the centre
  (22.4, 51.7, 80.9) measured, (21.4, 51.5, 80.6) by the formulas); take07's, over a ground that
  varies across the radar, within 7 (mean 3.2). The UI blends in linear light, where the same
  layers over a dark ground come out lighter (the disc's blue about 14 levels), so each layer is
  drawn with the game's alpha and the colour that makes the linear blend equal the game's over a
  reference ground (`GROUND`, 0.15: the takes' dark grounds), solved layer on layer: over the
  takes' grounds and the tutorial's grass the three passes are within 2 levels of the game's
  blend; over a brighter ground, e34 by day and the test map's floor, they part further: 4-5 on
  average over the zones by my model of the console's blend (3.5-4.8 with no correction), up to
  about 11 levels in a channel by the tester's. Both are estimates from a model of stored-value
  blending, not measurements (no take has those grounds). A
  demo approximation, not the game's; the game's stored-value blend for the whole HUD is shared
  machinery (play_scope_view.rs's passes are in those values) and isn't done here (#174's
  dimness is a different thing: the sampling of small art). Side by side with the takes
  (`BF_MAP=tutorial BF_TEST_GOTO=10.64,44.15,22.28,34.31` at frame 90 against take07 5.0 s: the
  grass is the same dark olive in both; `BF_MAP=e34` staged as the #33 takes against take03 1.0
  s, whose ground is dusk where the demo's is day): the zone colours of the tutorial frame
  against take07's differ by 3-10 levels (the demo's ground is a little brighter, (43, 46, 22)
  against (40, 34, 15)); a horizontal profile across the wedge 34 units above the middle has
  its edges between the same samples in both (the left one between 16 and 14 units out, the
  right between 14 and 16, samples 2 units apart), its plateau 113 against 116 levels.
* **Its scale** (FUN_000b0550): a world offset from the viewer, turned by the view, x 39 units
  (the max-range circle, the radar's +0x88 on a full screen) over the game options' `<radar>`
  max-range (120 m): 0.325 units a metre, each point cut to whole units. take07 agrees: the
  waypoint 17.4 m off drawn 5.4 units from the middle (5.65 by the code), the next, 55.5 m off,
  17.0 (18.0). The demo's 40 m across 44 units before #228 had no source.
* **The middle**: the viewer's dot, 180 180 180 (FUN_000ae7c0's colour for the HUD's own
  character, 0xae7f0: pale grey in take03 and take07; it was green). The code has no mode test:
  the campaign recordings' green middle is the squadmates' dots drawn over it, a few metres
  off at 0.325 units a metre.
* **The squad**: its living members in the world as green blips (FUN_000ae7c0's friend, 90 200 40;
  take11's marines; they were yellow) at that scale, one beyond the range on the circle (the game
  draws a rim marker for it: FUN_000ae7c0 through FUN_000b0ad0, not done).
* **The faces' places** (#347): each slot's face texture is drawn from the radar panel's corner
  plus the table 0x3c8c68's entry (FUN_000c2d70 puts the slot's portrait widget, class 0xe9535a,
  there: Tex (51, -15), Flint (112, 51), Brutus (51, 112), Hawk (-4, 51); `FACE_AT`), at its
  texture's size, the art packed into the top left of the 32 x 32 (Tex's from the second row), so
  the art boxes' top lefts are (107, 287), (168, 352), (107, 413) and (52, 352). They had been
  fitted by eye, (1.7, 3) too far right and down on average. Each face texture slid over take03's
  frames (1.0 s and 7.0 s, steps of a quarter unit, the art's luminance correlated with the
  take's) fits exactly there for Flint, Brutus and Hawk (0.88-0.94) and Tex within 0.5-0.75 up
  (his bracket over the art); the demo's own frame the same way: within 0.25 of the table for all
  four, and the whole radar region within 0.25-0.5 of take03's.
* **The portraits** (#303): a squad slot shows its face, its command tab and its health
  channel's fill only while a member is in the world in it (`slot_member`; a dead one keeps its
  skull: dead members are still in the squad); its channel's frame outline is drawn whether there
  is a member or not (#354: the game's channel draw tests only that its textures are bound). A slot
  whose member the zone doesn't have shows no more: the
  level spawns only the zone's members (its `h_ed54d255` names the others, [campaign.md](../campaign.md)),
  so the tutorial and m01_a (h_ed54d255 "brutus hawk flint") show Tex alone, as take04 does
  (`todo/28-tutorial`, 60 s: Tex's face, tab and channel, the other three places dark, their
  channels bare outlines), and m01_b
  ("hawk flint") Tex and Brutus. A member out of the world (`absent`: waiting for its spawn-trigger,
  or its copy taken away by FUN_00209900) counts as missing too. In the game the HUD's squad list
  (five entries at +0x34, one per slot by the type's +0x150) is kept each frame by FUN_000bd790:
  FUN_000c8570 drops the characters gone, FUN_000c84e0 adds the new ones (which list it reads and
  that the portraits draw from it aren't traced). The test map's emptied slots (#302) are the same
  rule. Checked (30 frames each, standing at the leader's spawn): `BF_MAP=tutorial
  BF_TEST_GOTO=46,-20,46,-20` and `BF_MAP=m01_a BF_TEST_GOTO=79.6,-100.5,79.6,-100.5` Tex alone,
  as take04; `BF_MAP=m01_b BF_TEST_GOTO=24.4,-10.0,24.4,-10.0` Tex and Brutus; `BF_MAP=e40
  BF_TEST_GOTO=-18.9,168.8,-18.9,168.8` and `BF_MAP=mp1` the four, as before; `BF_MAP=mp1
  BF_TEST_DIE=1` (120 frames) Tex's skull at the top through the hand-over; `BF_DEATHMATCH=1` no
  squad HUD, as before; `--test BF_TEST_GOTO=0,8,0,7.5` Tex alone, and with `BF_TEST_SQUAD="shrike
  soldier@m09_a,flint" BF_TEST_KILL=1` the Shrike's "?" in Brutus's place, Flint's skull, Hawk's
  place dark (#302's, unchanged).
* **The portrait box and the skulls** (#342). Every portrait and every skull is drawn whole in
  one **32 x 32** box (`PORTRAIT_BOX`, HUD units): the portrait widget 0xc3550 sets its size to 32
  x 32 in its constructor and its draw FUN_000bb300 sends that width and height with UV 0..1,
  never reading the texture's size (every portrait and skull texture in the data is 32 x 32, so
  they draw 1:1; the "wide" Brutus skull is the drawing inside its canvas). The box's place is
  the game's own (#347): the radar panel's corner (56, 301) plus the table 0x3c8c68's entry,
  `FACE_AT` (Brutus (51, 112), Flint (112, 51), Hawk (-4, 51), Tex (51, -15)); #342 had it from
  the slot's measured face centre, within a unit or two of the table against a panel corner at
  (57.5, 305) that was a guess, and that `portrait_corner` and the cropped-art inset it needed
  are gone. Another type's portrait or skull sits in the same box rather than being centred on
  its own opaque pixels (a skull used to be stretched to the face's box, 21 x 27 for Flint's).
  `PORTRAIT_CENTRE`, the centre of each slot's own face's opaque box, stays as constants for
  what points at a portrait (the chevrons, the "?").
  A dead member shows **its type's** skull (`CharacterType::skull`, h_00c51907): there is no feral
  or race skull in the data, only the four slot skulls (h_fe6fa2af slot 1, h_18343e47 slot 2,
  h_19da7869 slot 3, the wide one, h_17bd5b5c slot 4), which every predefined squad's types
  carry (a type reads non-zero for all 20 sdm types and the campaign four), so a dead feral shows
  the skull of the slot it holds. FUN_000c3590 takes type+0x158 dead and type+0x154 alive and
  hides the widget when there is no texture, with no fallback by race, name or slot: so a type
  with a portrait and no skull (the tutorial's marines) leaves its place **blank** once dead; the
  demo's old human and Brutus skull fallbacks are gone. Checked (`--test BF_TEST_FULL_SQUAD=1
  BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_PRESET=0.5,<squad>`, 45 frames alive, then
  `BF_TEST_KILL=0|1|2` for 120 frames and `BF_TEST_DIE=2` for Tex, each of the six squads: campaign,
  sdm_bf, sdm_feral, sdm_shrike, sdm_ferg, sdm_seers): every portrait whole and unsquashed in its
  place, each dead place showing its slot's skull (Brutus's the wide one at the bottom in every
  squad, the dead feral and Shrike Heavy too, Tex's at the top with the red chevrons as control
  passes); sdm_seers' slot 2 wouldn't die in 22 s (330 frames: not the HUD's), so that one is
  checked by the other squads' slot 2 and the data. `BF_TEST_SQUAD="Sgt. Gonzales@e01,Private
  Cogburn@e01,Corporal Stout@e05" BF_TEST_KILL=0|1|2` (the marines, with portraits and no skull):
  each place goes blank when its marine dies. Against the campaign footage on e34 (`todo/reference/
  Friendly Fire 2 + Death Cam.mp4`, 31.5 s, Brutus's skull at the bottom; the product owner
  confirmed it is campaign, not squad deathmatch) the skull's size and
  place agree to a couple of units. Its colour (#351): the game's skull is dark neutral grey, the
  terrain showing through its alpha of 170 and no fill behind it (the footage's notch beside the
  skull is the frame's own box). The demo's dark-blue notch fills (`NOTCHES`,
  `SquadPart::Notch`, a fit from #302's neighbourhood: the code has no fill widget in the
  portrait's path, FUN_000c3590 / FUN_000bb300 draw only the texture) are hidden while their slot
  shows a skull. **The notch table is by character** (Brutus bottom, Flint right, Hawk left, Tex
  top, as `FACE_AT` and `looks`): the first version had them in the place order top, right,
  bottom, left, so a death hid a living member's fill and left the dead one's; the test
  `notches_are_by_character` pairs each box with its portrait's. Checked with `--test
  BF_TEST_FULL_SQUAD=1 BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_PRESET=0.5,brute` and `BF_TEST_KILL=0`
  (Brutus, the bottom skull: the footage's case), `=1` (Flint), `=2` (Hawk; Brutus dies in that
  run too) and `BF_TEST_DIE=2` (Tex), 90 frames, frame 85, branch against an older main build
  (the dead slot's portrait box and notch box differ, the living ones' boxes are bit-identical,
  max difference 0; with Tex dead the bottom notch differs by the selected face's chevrons, which
  main draws at another brightness: the fill itself is there in both) and with nobody dead
  (nothing differs).
  **The skull's colour is the code's own** (#354; #351 had fitted it, 0.40, 0.46, 0.43, because "the
  colour path that darkens it isn't found"): the portrait's flash part (**The portraits' parts**,
  below) sets the portrait's colour bytes to (128, 128, 128) while the member's health is 0
  (0xc452f-0xc453a), and the sprite draw multiplies the texture by them (equal bytes: the plain
  program, no tint), so the skull is its texture at 128 / 255 (`SKULL_TINT`). The console
  multiplies and blends in stored values and the demo's UI in linear light, so the same 128 reads
  a little lighter here, which #351's fit absorbed. Measured on #351's footage's case (e34,
  `BF_TEST_HURT=brutus,500,0.5`, the recipe below, frame 50 of 60 at 15 fps; the core is the
  opaque texels eroded by three, and the render is put on the footage's ground the way the
  tester's method does, by inverting the demo's blend on the strip of ground under the box): the
  render's core (64.3, 66.0, 56.9) over its ground (86.7, 79.1, 55.2) is (52.6, 54.8, 52.8) over
  the footage's (59.6, 47.3, 41.2), against the footage's (49.2, 52.2, 47.6): 3.4, 2.6 and 5.2
  levels lighter (the fit: within 0.5 by construction; 0.53 by multiply: 8-12 out, the tester's).
  Kept the code's value: a skull 2% lighter, from the blend, with no fitted number left.
* **The portraits' parts** (#354, #355; the rules, tables and tint in `play_portrait_parts.rs`, the
  nodes in `play_hud.rs`): FUN_000c3680 gives each slot's portrait widget five parts, which update
  in this order: the **health flash** (class 0x1f0aff78, vtable 0x39fa44, made by 0xb9e80: update
  0xc43b0), the **selection tint** (0xfc8a6dfb, 0xc4280: the orange, #319) and the three
  **icons** (the speech bubble 0x1c8134b1, the "!" 0x07ccab64, the scope ring 0x11fb2340: base
  0xc4550); the slot's **health channel** (class 0x2a9b279: 0xc3080, 0xc32c0, 0xc3390) is a widget
  of the radar panel's. Every slot has them, the controlled character's too.
  * **The health flash** (`Flash`): +0x34 the member's health at the last update (0 as attached),
    +0x38 the low-health threshold 0.2, +0x3c a level. Each HUD update (`update_flashes`, on the
    HUD's clock `hud_step`, as the tabs' flash: it runs through the Squad Command menu's and the
    item list's pauses): health 0: the level 0; health under the last update's: the level at
    least **1.5** (a hit: any fall between two updates); otherwise health / most under **0.2** with
    the level 0: the level **1.0**. The portrait's colour bytes are then white, or (255, a, a)
    while the level is over **0.5** (a from 0xc4330: 128, and while a hit's level is over 1.0,
    255 - trunc(128 (1 - x^3)) with x = 2 (level - 1): 255 at the hit falling to 143 in a quarter
    second and 128 as the level reaches 1.0; never under 128 but for one frame at most, 127 while
    the level is within 0.0016 above 1.0, where the float product rounds to 128.0 before it is
    truncated), the level then counting down by dt to 0, and (128, 128, 128) when the
    health is 0 (the skull, above). So **a hit** reddens the face for 1.0 s (its first frame, a of
    255, is white: the first visible change is the next) and leaves 0.5 s of normal; **under 20%**
    the level restarts the update after it reaches 0: red 0.5 s, normal 0.5 s, the period 1 s (to
    the frame: 30 frames at 30 fps), until the member heals or dies, the controlled character's
    slot the same; a hit under 20% runs its 1.5 s into the blink; one that leaves the member over
    a fifth just ends. The selection tint updates after it and writes the colour last, so a member
    selected for the Squad Command menu keeps its orange (inferred from the order; not filmed).
    `BF_FLASH_LOG=1` prints each slot's part when it changes (the sim time, +0x34, +0x3c, the
    portrait's colour, the channel's frame and the fill's whiteness).
  * **The tint program.** Every sprite is drawn by FUN_000ba950 with a colour (red, green, blue
    bytes), and FUN_000ba820 picks its pixel program by it, one of the table 0x3e2530's
    D3DPIXELSHADERDEFs (FUN_000a4d00): with **equal bytes** (the white of every plain sprite, the
    dead skull's grey) technique 0x25 (0x3e0230), texture x colour in stored values and then a
    push toward white by the widget's +0x14 (0 as made); with **any difference** technique 0x23
    (0x3e0140), whose five combiners read: grey = min(1, T.r^2 + T.g^2 + T.b^2) (the texel's dot
    product with itself), then colour x grey, with the texel's alpha (and a (x - 0.25) x s + 0.25
    round it that does nothing for the s = 1 of every widget here). So the flash's red face, the
    icons' blue-white and the channel's red frame are not a multiply: a red channel that a
    multiply leaves where it was rises, and the texture's contrast is squared. `tint_texels` bakes
    it into an image (the faces, the frame's red copy, the icons; `flashed_face` caches a face by
    its green byte). Checked texel by texel on the face texture and the video, the ground taken
    from the rest frame: Hawk's steady red (a 128) on screen is (143, 74, 77) in #36 take01
    (17.5-18.3 s) against the program's (142.9, 77.5, 79.9) (a multiply gives (98.6, 55.4, 67.6),
    a luminance tint x 1.38 gives (134.6, 74.2, 76.6)); Tex's, in #354 take02: (108, 56.7, 59.5)
    against (107.8, 56.5, 59.0), and the first frames of the hit (a near 232-238) (106.8, 95.5,
    99.5) against (107.8, 96.7, 99.2). It
    is very likely what the fits of #319's selected faces (`SELECTED_GAIN`, "luminance x colour x
    1.28") and of the top-left bars' flash colours (`FLASH_TINT`, "x 1.64") stand in for (not
    checked, not replaced here).
  * **The health channel** (`CHANNELS`, `SquadPart::ChannelFrame` / `ChannelFill`): two 32 x 32
    sprites in the table 0x3c8da8's rect for the slot, from the radar panel's corner: Tex (80, 12),
    Hawk (13, 21), Flint (115, 108) and Brutus (48, 116) the last two mirrored (a negative size),
    each a diagonal between its portrait and the next. The **frame** (h_e0706e07 for slots 1 and
    3, h_00ff5cfb for 2 and 4: a blue outline at alpha 136-170) is drawn white, or
    (160, 32, 32) (FUN_000b8820) while the slot's level is over 0.5, through the program: a flat
    red outline. The textures' **clear texels are white** (255, 255, 255, alpha 0); the console draws
    the sprite a texel to a pixel, so that white never shows, but the demo draws it 1.5 pixels to
    the unit through a bilinear filter that blends colour and alpha separately, and drawn as it came
    the outline's edge turned a pale lavender halo (Hawk's empty channel in the tutorial: the
    outline's mean (88.6, 118.2, 173.8) against take04's (38.1, 69.8, 123.7), the red 50 over).
    `portrait::bleed` gives each clear texel beside the outline its neighbours' mean colour before
    the image is made (`frame_images`, for the white copy and, through the tint program, the red
    one). The **fill** (h_e684da10 / h_1cee1256: a gradient of about 20 levels in its blue
    channel, white at the portrait's end) is drawn in **(192, 87, 94)** by the program at
    0x3e0050 (technique 0x22, which the sprite draw takes when its twelfth argument is 1, and only
    this draw sets it; its constant is the channel's +0x34 through the vtable getter 0x10fa30): a
    texel shows, in the flat colour at its own alpha, while its blue is at least the **empty
    byte**, round(255 x (1 - floor(100 x health / most) / 100)) (FUN_000c32c0), so the health
    left grows out from the portrait's end in the gradient's steps (`fill_images` makes one image
    per level; the demo's old fill was the radar frame's dark texels, fitted); a member with no
    health shows none (the brightest texel's blue is 249). In a hit's first half second the fill is
    pushed toward white by (a - 128) / 128 of the way (`pushed`: (254, 253, 253) at the hit's
    frame, 0xc3449): the pale flash. The **frame is drawn for every slot**, with a member in the
    world or none: the channel's draw (0xc3390) tests only that its two textures are bound
    (0xc3399-0xc33a9), and the member sets only +0x34, which is 1.0 with none, so it is the fill
    that goes empty (the tutorial's take04 at 60 s: Tex alone, the other three channels bare
    outlines). An empty slot's flash part isn't updated (0xc43e8), so its frame is white; the fill
    is only for a slot with a member.
  * **The icons** (`ICON_AT`, `SquadPart::Icon`): 16 x 16, in (192, 215, 255) through the program
    (the texture's white body gives it exactly), at the table 0x3c8ee8's entries from the radar
    panel's corner, three a slot in this order: Tex (45, 20) (57, 20) (68, 20), Flint (93, 40)
    (93, 56) (93, 72), Brutus (45, 92) (57, 92) (68, 92), Hawk (20, 40) (20, 56) (20, 75). The
    speech bubble (h_e064867a) shows while the member's voice line plays (FUN_00126810:
    `Player::speaking`), the "!" (h_018b0dee) while its special skill is on (the skill's +0x1c bit 0
    and an owner: `Skill::on`), the ring (h_1d870a5c) while it is scoped (char+0x7a8:
    `scope_level`), each only for a living member; the bubble and the ring also only on the
    full-size HUD, which is all the demo has (the split screen's places are the table's last four
    rows, not done). The game has no icon for a line's priority or tag: these are the three.
  * **Verified.** Against `todo/354-portrait-health-flash` (xemu, the flash part watched at
    20 Hz): the level jumps to 1.465-1.473 at a hit (1.5 less a frame), counts down at 1 a second,
    reaches 0 1.5 s after it and, with the member under a fifth, restarts at 1.0 the update after
    (take05: Tex 115 to 15, 0 at gt 3.062, 0.957 on the next row), and held under a fifth restarts
    every 0.997 s on average (take01: Hawk at 10 of 65, 11 restarts from gt 2.36 to 12.33; take02:
    Tex at 20 of 115); a hit that leaves 30% (take03: 115 to 75 to 35; take04: Hawk 65 to 45 to 25)
    ends after the 1.5 s. The demo: `BF_FLASH_LOG=1 BF_TEST_HURT=hawk,55,0.5` (10 of 65) logs 1.467 at
    the hit and restarts every 0.999 s of its frames (30 fps capture: 1.0 s, the frame
    included); `BF_TEST_HEALTH=20` (Tex at 20 of 115) blinks red 15 frames, normal 15 (30 fps);
    `BF_TEST_HURT=tex,100,1.0` runs the hit into the blink (the level 0 after the hit's 1.5 s of
    frames, red again the update after, as take05); `BF_TEST_HURT="tex,40,1.0,5.0"` (115, 75, 35)
    ends both flashes with no blink.
    Pictures, e34 as the takes (`BF_MAP=e34 BF_TEST_GOTO=-83.39,-82.79,-83.39,-82.79
    BF_TEST_GO_AT=999 BF_TEST_NO_HOSTILES=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05
    BF_CAPTURE_FPS=30`), the demo's crop against the video's at the same pixels: Hawk at 52% (take01
    16.8 s) and 32% (17.5 s, red): the fill's shape and length the same, its colour (192, 87, 94)
    against the video's (189.2, 85.7, 92.6); the hit's first frame the fill white-pink (243, 223,
    224 for the frame at level 1.467; the video's first frame after it (239, 207, 211), a frame
    later in its own clock) and the channel's frame red; the face's steps white, pale, pink, red
    as the take's frames 17.43-17.57 s. The demo's faces read 15-26 levels lighter than the video's
    at rest and in the flash alike (the notch fill behind them, `NOTCHES`, and the linear-light
    blend: not this ticket's), the change from rest to red the same to within 20%: Hawk (+52, -38,
    -56) against (+44, -31, -50), Tex (+47, -24, -51) against (+38, -22, -48). The dead skull:
    above. The icons on e34 (`BF_CHARACTER=brutus BF_TEST_SKILL=1.0,3.0
    BF_TEST_SQUAD_SKILL=hawk,1.5 BF_TEST_SPEAK="2.0,hawk,f9f8e847;2.5,tex,f71090f4;3.0,flint,e50e45c4"`;
    Flint's ring: `BF_CHARACTER=flint BF_TEST_SCOPE_PRESS=0.5,3.0,4.5`) against #35 take18 12 s
    (Hawk's "!", her second place, under the absent bubble), take06 5 s (Brutus's, his own) and
    #62 take06 6 s (Flint's ring, her third): the same places to the pixel, the same pale blue;
    Brutus's "!" is gone once Vengar is off (3.13 s) and the ring once the third press leaves the
    scope (4.53 s), as the skill and the scope level go; all three are hidden for a dead member by
    the game's own test (`!u.dead`; the code's, not filmed).
    The frames of the empty channels, and a bright ground (the demo blurred to the video's
    softness, a Gaussian of 1.2 px, the mean over each frame sprite's 64 outline texels). The
    tutorial with Tex alone (`BF_MAP=tutorial BF_TEST_GOTO=46,-20,46,-20`, frame 15 of 20) against
    take04's 60 s, where the other three channels are bare outlines: Hawk's (37.4, 75.9, 137.1)
    against the video's (38.1, 69.8, 123.7), Flint's (33.0, 90.0, 143.6) against (37.9, 82.9, 130.4),
    Brutus's (35.5, 86.8, 134.0) against (43.6, 87.1, 127.0). Main, with no frame sprite (the radar
    frame's own baked outline alone), reads (33.6, 64.5, 114.1), (25.9, 79.4, 117.1) and (31.9, 66.7,
    102.2); the branch before the bleed (75.2, 97.2, 142.8), (70.3, 108.3, 148.9), (72.5, 102.9,
    137.9). The test map's emptied slots show theirs too (`--test BF_TEST_FULL_SQUAD=1
    BF_TEST_GOTO=0,8,0,8 BF_TEST_SQUAD_SLOT="0.5,1,empty;0.5,2,empty"`, frame 44 of 45: Flint's and
    Hawk's channels dark navy with their blue outlines, Tex's and Brutus's pink). Over the
    yellow lava of e13 (`BF_MAP=e13 BF_TEST_GOTO=30,30.5,30,40.5 BF_TEST_GO_AT=999
    BF_TEST_NO_HOSTILES=1 BF_CAMERA_PITCH=-0.06 BF_TEST_HEALTH=35`, Tex at 30%) against take06's
    still, the ground about (255, 209, 33): the outline (117.6, 151.8, 126.8) in the footage,
    (159.9, 142.9, 141.7) in the demo (main's (188.9, 145.3, 114.8)); the fill's flat pink (187.3,
    86.3, 89.5) against (191.2, 87.7, 93.0) (main's (244.6, 115.0, 106.8)); and the channel's
    **empty interior**, which is the radar frame's own: (87.1, 86.5, 45.0) in the footage against
    (156.3, 127.4, 48.1) in the demo and (160.0, 127.6, 50.8) in main. The frame's navy there, (0,
    3.8, 52.5) at alpha 0.667, blended over that ground in stored values (the console's way) gives
    (85, 72, 46), and in linear light (the demo's UI) (156, 127, 47): the whole difference, and the
    same before this ticket. A bright ground shows it, the radar's translucent panel (and the
    outline over it) reading lighter than the console's, by 70 in red on lava; its fix belongs to
    the HUD's blend, not the portraits' parts.
  * **Not done or not filmed:** the channel frame's red as measured (the takes show a dark purple-red
    outline; the demo's looks the same, a little pinker: #36 take01 at 17.9 s beside frame 36 of
    `BF_TEST_HURT=hawk,55,0.5`, seen, not measured); the radar's translucent panel over a bright
    ground (above); Flint's and Brutus's own flashes (the same code); a
    flagged member under a fifth (orange by the update order); the split screen's rects, textures
    and icon rows; the yellow corner brackets (part 0x16b9cec9, FUN_000c37a0); a full-screen red
    vignette on the controlled character's hit, which is not this part.
* **The command tabs** (#333; `play_hud.rs` `update_tabs`, `TAB_FOLLOW` ...): the small icon in
  the box by each portrait is its member's order, the brain's command mode (squad.md, "Squad
  commands"). The tab widget (FUN_000c3d10, made one a portrait by the HUD's factory) sets it each
  frame from the slot's member's brain (+0x58; its update 0xc3f80's jump table 0xc40e0):

  | Mode (the order) | Icon | Size |
  |---|---|---|
  | 1 FOLLOW (Cover Me, and the start) | h_eeaa5549, green arrows | 16 x 8 |
  | 3 GUARD (Stand Ground) | h_fa29b7a4, yellow bars | 8 x 16 |
  | 4 WINGMAN (Fire At Will) | h_f6722366, a ring of red and orange dots | 16 x 16 |
  | 5 MOVE_OUT (Move To) | h_eed3270a, a blue arrow | 16 x 16 |
  | 2 FORMATION, any other | none: the tab hidden | |

  * **Where**: its top left at the table 0x3c8ea8 (FUN_000c3f40, by slot) from the radar panel's
    corner, the icon at its texture's size: Tex (38, 5), Flint (115, 37), Brutus (82, 114), Hawk
    (5, 81). The art fills each texture's top left. The code's corner is (56, 301)
    (FUN_000c2be0), and take03 and take08's icons start there (Hawk's arrows, bars and arrow all
    from (61, 382), Tex's arrows from (94, 306)). The radar frame, whose boxes hold the tabs, was
    drawn 1.5 right and 4 lower than that until #347 (`RADAR_FRAME_AT` (57.5, 305): take03 1.0 s
    has Tex's box's edges at x 90.7 / 104.7, y 302.7 / 317.3, the demo's were at 92.0 / 106.0,
    306.0 / 320.7); the frame, the tabs and the brackets all go from (56, 301) now, each in its
    box as in the takes. The panel slides with the menu, the tabs with it. Before #333 every tab
    drew the follow arrows squeezed into a 10 x 10 box at a measured centre.
  * **Who**: every slot's member in the world, the controlled character's too (take03: Tex's
    arrows), hidden while it's dead (0xc40bb).
  * **The flash**: a new icon (the update's texture compare, the first one set at a map's start
    too) starts a flash of 0.67 s (1.5 a second, FUN_000c4100, on the HUD's clock: it runs through
    the menu's fade-out): the draw (0xc4130) puts a copy of the icon under it, scaled by 1 + 1.8 t
    and shifted by -1.8 (s - 1) x s each way, at alpha clamp(2 (1 - t)^2, 0, 1): whole for its
    first 0.2 s, 2.8 times the icon and gone at the end (the alpha raised to 2.2 here, the HUD's
    rule for the console's sRGB blending). take03: the bars a 9.3 x 12 blob at 5.35-5.55 s
    against their 7 x 9.3 after; the arrows bold at 14.2-14.3 s after Cover Me (the "green
    flash").
  * `BF_ORDER_LOG=1` prints each tab's change of icon.
  * **The speech icons** (#332) are the first of the portrait's three icons, now placed from the
    game's table: **The portraits' parts**, below. #332 measured their places from the takes
    before the table was found (the icon's white body, every channel over 185, at 1.5 px a unit:
    Hawk take01 17.6 s x 78.0-89.3 / y 344.7-352.0, Flint take08 4.9 s x 151.3-162.7 / y
    344.7-351.3, Tex take12 3.9 s x 105.3-114.7 / y 324.7-332.7, Brutus take12 12.6 s x
    103.3-114.7 / y 397.3-403.3), and the table's first entries agree with them to a unit.
  * Checked (e34 staged as the #33 takes, `BF_CAPTURE` at 15 fps: `BF_MAP=e34
    BF_TEST_GOTO=-83.39,-82.79,-70.8,-79.5,27 BF_TEST_GO_AT=999 BF_TEST_NO_HOSTILES=1
    BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05 BF_TEST_CURSOR_AT=-70.8,-79.5 BF_ORDER_LOG=1
    "BF_TEST_COMMAND=2:hawk:0.4;2.9:brutus;3.5:flint;4.1:brutus;5:stand;11:hawk:0.4;11.9:brutus;12.5:flint;13.5:cover;17:hawk:0.4;17.9:brutus;18.5:flint;19.5:fire;23:hawk:0.4;24:move;26:accept"`,
    take03's presses, then Fire At Will to all three and take08's Move To for Hawk): Hawk's and
    Flint's bars from the Stand Ground frame, their copies grown over the next three as the radar
    slides home (frames 76-79), as take03 7.0 s at rest (frame 100); the arrows back after Cover
    Me, Brutus's unflashed (he was in FOLLOW), as take03 15.0 s (frame 235); the three rings
    after Fire At Will (frame 330, as #33 take05 from 5.4 s); Hawk's blue arrow after the cursor's accept, as
    take08 27.0 s (frame 398); her bars on reaching the point, the copy growing over frames
    405-410, as take08 33.0 s (frame 430). Each icon in its box as in the takes, the radar group
    1.5 right and 4 lower (see "Where"). No orders (the same recipe on main's build, frames 1-32):
    nothing differs but the four tab boxes (the largest other difference 19 levels: grass), where
    the arrows are now the texture's 16 x 8 at the code's place, and each tab's first icon
    flashes in the first 0.67 s (frames 1-5).
* **The other characters** (FUN_000ae7c0, radar-contact 1, or 6 for a boss): one within the range a
  record as an objective's (below), a friend green 90 200 40 (40 100 20), one whose team the
  viewer's is hostile to red 200 40 90 (100 20 40: take08's and take11's terrorists), anyone else
  70 70 70. A hostile shows only once the viewer or a living squadmate knows of it (FUN_000ae580 ->
  FUN_000ae470, with h_14be5318 off as shipped; `radar_knows`): the controlled character's own
  sight (`Sensor::scan` from it, each one seen kept 15 s unseen, as a contact is: its sensor keeps
  none, #197) or a squadmate's contacts (the squad's members and the marines following; the
  squad's broadcast tells them each other's targets). Every character shows only within the
  player's character type's `radar-range` (type+0x2a0, 60 m for the four and the marines: the block
  h_12501487 beside `<combat-skill>`, FUN_0018fde0), across (FUN_000b0670), fading over its last
  2 m, (reach - d) / 2 (FUN_000b0770); its easing toward h_f0245e27 by char+0x57c isn't done
  (char+0x57c taken as 0: its writers aren't traced beyond the character's set-up). The squad's
  own blips within the max-range show only within that reach too. (Before #228 the demo drew the
  first three of them as yellow squad blips.)
* **The objectives**: the objects and characters with radar-contact 2, 3 or 7 (OBJECTIVE,
  OBJECTIVE_NO_CURSOR, IMPORTANT_PICKUP: FUN_000ae580 hands those to FUN_000aece0), which the
  level gives them or a radar-trigger sets ([triggers.md](../triggers/radar-triggers.md), "Radar-triggers"). Each
  game frame (30 a second) each one within the max-range gets a record at its place (FUN_000aece0:
  1.0 s of life when it wasn't seen in the last 3 s, FUN_000ae360, else 0.3 s); the records lose
  the frame's time and grow by 8 units a second from 1 unit each way (FUN_000ae3f0), and each is
  drawn as the dot texture h_ecea7a68 (an 8 x 8 soft disc) that many units each way round its
  point, yellow (255 255 0; the important pickup's 30 90 30), its alpha x min(life / 0.25, 1)
  (0xb01b0). So a dot is solid while its object is on the radar and fades over 0.2-0.3 s once it
  isn't, and a new one comes in with a 1 s record. While its object's blink time (+0x10c, the
  radar-trigger's, counted down by the object) is above 0, its records are drawn dim (alpha
  255 >> 6) on the half of each 2 s where frac(t / 2) < 0.5 (0xaedd5): a dot set with time 2
  shows for its first second (and the 0.3 s its last full records take to fade), is gone until
  2 s, then stays. One beyond the range is a rim marker, an "L" with its corner on the circle:
  FUN_000b0ad0 gives the point on the circle and one 5 units in, and the draw (0xafaa0) runs two
  lines from the circle point to the inner one pushed out 5 units to each side, square to the way
  in (arms 7.1 units long); 1.33 units wide here, the takes' ~2 px at 720p (the game's lines are a
  pixel of its 640 x 480). It was drawn inverted, its corner inside, before the fix round.
* The dots and markers are drawn on the CPU into an image over the radar (256 x 256, two pixels a
  unit), redrawn each frame while there's any.
* The circles' and the wedge's layers (128 x 128 each) are redrawn when their inputs change:
  about 1 ms on the CPU (0.97 ms in the test build for three circles and the viewer's pass) and
  two 64 KB uploads, on nearly every frame while the camera turns or a member moves (the points
  are whole units from the view's heading), none while all stand still. They go with the map
  (`radar_disc::forget` on leaving it; `setup_hud` drops them for a map with no squad radar), so
  a deathmatch map after a squad one redraws nothing (#347's review).
* **Under the Squad Command menu** (#319, play_squad_menu.rs; [squad.md](../squad/squad-command-menu.md), "The Squad
  Command menu"): a member selected for an order is drawn 200 200 200 on the radar (FUN_000ae7c0
  for one FUN_000c86d0 says is flagged), its portrait in the selection's orange (0xc4280's ff a0
  6a: the face's brightness in it, a fit to take01 and take03, where a plain multiply leaves a
  blue face grey: `selected_face`) and bracketed, the bracket closed toward the radar (the part
  0x1207a43b: h_ff5d4d4f above and below, h_fdec2fc6 left and right, mirrored by 0x3c8ca8's
  negative sizes: `SELECT_FRAMES`). A d-pad press flashes the three chevrons at the portrait
  pressed, the outer two fading first, the last gone 0.25 s on (FUN_000c4dd0, FUN_000c4cd0). The
  radar, its portraits and what's on them are one panel (the game's widget 0xc2d40,
  `RadarPanel`) that slides from its corner to the screen's middle less its width and 20 as the
  menu fades in, 1 - (1 - fade / 0.3)^3 of the way (FUN_000c2be0: 116 right and 120 up), and
  back as it fades out; it draws over the menu's darkening backdrop (`RADAR_Z`). Checked against
  take01 (4.0 s) and take03 (3.9 s, three selected): the faces, brackets and the pale middle
  dot, the panel at the take's place (the frame texture's edges fit take01's open menu at
  (171.6, 181.0), the code's slid corner (172, 181), the demo's frame (172.0, 180.5); it was
  within 2-4 units before #347 moved the corner). The chevrons' stack is the demo's own fit
  (`CHEVRON_GAP`): the game's flash widget (0xf5c0f471) draws the textures h_016c3d13 (Flint and
  Hawk, 16 x 32, mirrored for Flint) and h_091adf04 (Tex and Brutus, 32 x 16, mirrored for
  Brutus) in the boxes of the table 0x3c8e28 ((26, 49, 16, 32) Hawk, (103, 49, -16, 32) Flint,
  (48, 23, 32, 16) Tex, (48, 105, 32, -16) Brutus from the panel's corner), each of the three
  6 units further out (FUN_000c4bc0: (2 - n) x 6), tinted by their alpha (0xc4c37: the red
  (a, a / 4, a / 4), not traced further); the demo's are rotated copies of h_158f87c1, 2-3 units
  off take02's taps at Hawk (2.20 s) and Tex (5.70 s) after the fit, and smaller (not done).

Verified (the tutorial, `BF_TEST_GOTO=10.64,44.15,22.28,34.31` into the first patrol's circle,
[triggers.md](../triggers/test-hooks.md) "Test hooks"; crops of the radar against take07 and take03): the first
waypoint's dot is gone from ~1.3 s to 2 s (frame 24) and stays; at the circle (2.87 s) the new
one comes ahead, holds ~1.1 s (frames 43-60), is gone ~0.75 s (frames 62-72) and is back for
good at 4.93 s, and the old one fades in 0.2 s, as take07's (in 1.1 s, gone 0.8 s); their
distances from the middle at take07's scale. sdm_e34 (regression, `BF_TEST_GOTO=0,0,0,0`
against main's build): the squad's two blips behind Tex at half their old distance from the
middle, green, the middle grey; the map has no objectives. The base with a far objective
(`BF_TEST_GOTO=-36.4,37.3,-57.3,22.9 "BF_TEST_RADAR=h_1d422d6d,3"`: a mine 184 m off): the far
mine's "L" on the circle with its corner out, its first record a growing dot on the circle. The
knowledge gate and reach, against the takes: at the first save circle (the walk above) no red
dot, the gate sentry being 74 m off and unseen (take07 has none); at the lava pit
(`BF_TEST_GOTO=48.1,-1.5,46.26,-25.92`) one red dot near the middle from frame 90, the second
building's guards (beyond the 60 m) and the other groups not (take08: one or two near the
middle); in the base walk one (terrorist_9 beside Tex), the base's other three, inside and unseen,
not (take12: one to three at a time). sdm_e34 with the test tools' NPCs (`--test BF_MAP=sdm_e34
BF_TEST_GOTO=-44.4,15.5,-44.6,15.1 "BF_TEST_NPC=0.5,brutus,enemy,fight,-44.5,5;0.5,flint,enemy,fight,-44.5,-55;0.5,flint,friend,dummy,-52,15;0.5,hawk,neutral,dummy,-38,22"`):
the near enemy, known to Flint, red; the far one (75 m) none; the friend green; the neutral 70 70 70.

**The red blip where the first patrol fires** (#245; take07 54.5-56.5 s). It is not a shot or noise
contact. The shipped options have h_14be5318 off (`<radar ... h_14be5318="false">`), and then
FUN_000ae580 shows a hostile only through FUN_000ae470, the world's pair table (world+0x1f8,
FUN_000572a0): the line-of-sight bit of the (viewer, hostile) pair, or a living squadmate's. Only
with the option on would a hostile show while it fires or moves (the weapon's +0x218 above 0, or
its velocity). A patrol member's blip therefore comes when it comes into sight, a moment before it
fires, which is what the take shows (the terrorist is off screen at 54.5 s, the first tracer at
55 s, the first hit at 56 s). It was already drawn once #228's knowledge gate (da53f24, 18 Oct)
landed, a day after the ticket was filed. Checked with `BF_MAP=tutorial BF_TEST_GOTO=44,91,32.67,93.7
BF_TEST_PANEL=a,1 BF_CAPTURE_EVERY=3` (Tex where take07 stands) against take07's radar: the blip
comes as a growing ping that fades over ~1 s to a steady dot, in the same place on the radar (39-42 m
out); its brightest pixel is (200, 40, 90) in both the demo's frames and the take's (take (196-209,
31-40, 84-93) after compression); the ping's red pixels peak at 287 (demo) and 241 (take), the
steady dot is 47 against 40. The one difference: the game's bit is the clear ray alone, with no
field of view and, beyond the reach where it isn't tested, stale (FUN_00057a30: the ray is cast only
while one has the other in reach), where the demo's gate is the controlled character's sight (cone
and range, kept 15 s) or a squadmate's contacts. They agree on the take; the stale bit is not done.

Not done: the radar's sound for a contact set with h_12b33046 (no tutorial trigger has it); the
dots' size (the game's character records are re-made each game frame and grow, so a dot is a
blob 5-7 units across, the demo's squad blips and middle dot 4); the squad's rim markers (above)
and their 200 200 200 when selected (FUN_000c86d0); the reach's easing by char+0x57c (above);
contact 2's cursor on the screen (FUN_00052db0); which list of characters the HUD's squad list
is kept from (FUN_000bd790, "The portraits" above); the disc's and the frame's colours blended as
the console does (stored values: see "The disc, the circles and the wedge"); the portraits'
notch fills behind the faces (the demo's blue boxes are brighter than the takes' dark navy: a
fit, #342's neighbourhood; hidden behind a skull since #351). A test map dummy is no AI target, so nobody knows of a hostile
dummy and it never shows.
