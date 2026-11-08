*Part of [play/](README.md), the former page `play.md`.*

### Ammo, reload and the crosshair (#114)

`src/bin/play_ammo.rs`, the HUD parts in `play_hud.rs`. Logic from `default.xbe` (functions cited
in the code), checked against the weapon takes in `todo/49-*` … `todo/68-*`.

* **The reserve is the squad's**, one count per ammo-type, shared by every member and every gun
  of that type (FUN_0022da80 looks the weapon's `ammo-type` up in the holder's inventory). The
  Foley 356 and the L-Shot share 11mm Ammo (type 1); the MK-ASLT and the Minigun High ROF Ammo
  (type 2). The panel reads "clip / reserve" (orange), e.g. "24 / 50" for Hawk's Foley, for a gun
  with a magazine prop (the others: **The panel's count and its red**, below).
  * Where a level starts the squad from its own placements (a spawn-trigger's template, or the
    squad placed in the world: [triggers.md](../triggers/README.md), #220; every campaign zone), the
    reserve is the ammo they carry (`<inventory><h_10dd6db2 name count>`) and **nothing of any
    other type** (#281): the squad's store at the start (see **Items: the squad's starting
    inventory** below), whose IG_AMMO stacks are the ammo-types. The tutorial's Tex 500 High ROF
    (take03's "48 / 500"); e34's 11mm 40, Sonic, Particle and Rail 200, every other type 0, as
    xemu read the game's store at a fresh start (#196 notes); e40's 11mm 40, High ROF 600, Sonic
    200, Shotgun 80, Cutter 120 (39 and 199 once the opening's two reloads have taken their
    round: see the code 100 in [cutscenes.md](../cutscenes.md)). A gun of a type nobody carries has
    no reserve, as in the game: #73's Therm Sweeper picked up on e34 read "6 / 0". The members'
    counts are pooled as the game pools them (#230): each member's own store holds at most a
    stack of an item (FUN_0014bfe0's clamp to the type's stack-limit, +0x180), and joining the
    squad merges it into the squad's one store with the same clamp. So m09_a's Brutus 200,
    Flint 120 and Tex 160 of Sonic Ammo (h_ef0c4c1e, stack-limit 200) make 200, not 480
    (`BF_AMMO_LOG`: "type 3 200").
  * A map without squad placements (the test map's floor, the deathmatch maps, whose sdm
    loadout is #144's) starts with a full stack of every type: the stack-limit of the level's
    ammo item (objecttypes `<inventory>` with function-type 14, IFSET_AMMO_BOX; its h_e5f51266 is
    the ammo-type): 11mm 50, High ROF 600, Sonic 200, Particle 200, Rail 400, Shotgun 80, Cutter
    120, Bio 200, Rocket 40, Energy 200 (the same in every level that defines them, so a level
    without the item uses those). The full stack is the demo's choice.
  * A zone whose placements carry no ammo at all (m03_b, m03_c, m08_b: the game's squad comes to
    them from the zone before, with the save game's pool) keeps the full stacks too: the demo's
    stopgap until zones carry the reserve over (#231), as with none no gun could reload there.
    A zone whose placements carry any ammo keeps the carried-only rule.
  * The test map's NPCs reload from a bottomless reserve of their own.
  * **The reserve grows by walking over the ammo boxes** (#18, [pickups.md](../pickups/ammo-boxes.md), "Ammo
    boxes"): a box's `pickup-amount` into the stack of its ammo-type up to the stack-limit, the
    panel's "clip / reserve" following in the same moment (take01: 38 / 100 -> 38 / 125), the
    line "25x Particle Ammo" on the HUD, or "Particle Ammo (full)" in red with the box staying
    when the stack is at its limit (the player's own character only: an AI squadmate on a full
    box shows nothing). `BF_TEST_AMMO=<ammo-type or ammo name>:<n>[,...]` sets the reserves at
    the start.
* **Recharging guns** (the LZR-10, -23 and -50: `h_1d1e0e9c` 2, 1.25 and 1 s a round) show
  "N Regen" (string `h_e246de1d`) and never reload. Their clip gains a round every ammo-regen
  seconds while it isn't full and the gun's own cooldown is out, held or stowed (FUN_0022ebc0),
  so the first round comes back cooldown + ammo-regen after the last shot (LZR-23: 1.65 s; the
  takes 1.6-1.7 s). Each gun keeps its own cooldown for this (the game's weapon +0x218), set to
  1 / rate by a shot, a dry click, and the start of a weapon switch on the gun put away
  (FUN_0011e130 starts the slot change, then FUN_0022dc00 on the held weapon). So a switch away
  delays the next round and the switch back doesn't, as in the takes (LZR-50 20.996 → 23.202 s
  with Y at 21.83; LZR-23 21.512 → 23.419 with Y at 21.7). The demo's own 0.1 s after a switch
  and 0.2 s gun raise don't touch it. This rule came in ahead of the LZR batch so an emptied
  LZR refills.
* **The panel's count and its red** (#408: FUN_000cbdd0, and the row's update 0xcb970 and draw
  0xcbbd0, [code-map/hud.md](../code-map/hud.md)). `play_hud.rs`: `ammo_text`, `infinite_clip`,
  `blink_step`, `AmmoBlink`, `tick_ammo_blink`, and the panel's two images `HudGen::panel`.
  * **The count follows the gun's kind.** With an ammo-regen "clip Regen"; with none but a
    magazine prop (the weapon type's `h_0bfde1e8`, `WeaponDef::magazine`) "clip / reserve"; with
    neither, **the clip alone**: the iKhan-GPL (its `h_0bfde1e8` is "") reads "16", "15" ... "0"
    (todo/55 take01 2.0 s "16", 4.0 s "15", 6.0 s "14"; take02 1.0 s "3", 6.5 s "0"), no slash and
    no reserve figure (its ammo-type 9 has no item in any level). **A clip of -1 writes
    nothing**: the creatures' natural weapons (a Seer's Psionic Blast, a hound's Breath Attack,
    the four turret guns; `ammo-count` -1) show the icon over no number and are never red. The
    demo keeps such a clip at 1 (`ammo.max(1)`), so the panel asks the definition for the game's -1.
    Of the guns a squad member can hold only the iKhan-GPL has no prop and no regen; the type-6
    blasts are never held. Not filmed: a player-controlled Seer or hound, so their box is from the code.
  * **The red follows a timer per weapon slot**, the game's row timer +0x5c from the clip over the
    gun's size (`blink_step`): at 0 it is set to 2.0 every frame (steady red); up to a tenth of the
    clip (0.1 included: 10 of 100) it is set to 2.0 whenever it is at or below 1.0, a 1 s cycle; up
    to a fifth (20 of 100) only at or below 0, a 2 s cycle; above that it is cleared; then it runs
    down by the frame's time. The panel is red while it is above 1.75, the first 0.25 s of each cycle.
    At the game's 30 frames a second, in single precision, the cycles are 61 and 31 frames (2.033 s
    and 1.033 s) and a red is 7 frames (0.233 s). The rows are the character's weapon slots
    (FUN_0014d330 / FUN_0014d3b0), so a gun's timer goes with it through a switch; every slot's
    timer runs, shown or not (assumed: whether the panel updates hidden rows isn't traced). A row with no
    gun (a weapon pickup's release to its grab) takes no share and only runs down. It is the same
    for every gun: the LZRs' "N Regen" blink at a fifth too.
  * **The red is the sprite draw's tint program, not a multiply.** The row's draw gives the panel
    FUN_000b8820's a0 20 20, and a colour with unequal channels sends the sprite draw to the program
    at 0x3e0140 (the colour x min(1, r^2 + g^2 + b^2) of the texel, its alpha kept: the channel
    frames' red uses it, #342), so the red panel is the texture's texels through it, made once
    (`tinted`) and swapped in for the white. Multiplying the blue texture by a0 20 20 leaves a
    near-black panel (the first try of #408); predicting take02's red state (7.0 s) from its white one
    (1.0 s) over the same background, the program is within (+3, -1, -4) levels of the measured
    mean (64, 41, 42), 11 RMS, a multiply by a0 20 20 (-21, -4, -2) and 18 off, and the fit the demo
    had before, a multiply by (1.0, 0.35, 0.3), (-15, +3, +10) and 16 off.
  * **Checked** (15 or 30 fps captures, `BF_MUTE=1 BF_BACKGROUND=1`, the panel's crops beside the takes'):
    * `hud::tests` (`cargo test --bin bf_play hud::tests`): the count's forms, the cycles (61, 31
      frames, 7-frame reds), steady at 0, none above a fifth, take02's clip sequence, a row with no gun.
    * The iKhan-GPL, `BF_MAP=sdm_e34 BF_CHARACTER=0 BF_TEST_GOTO=-44.4,15.5,-44.4,15.5
      BF_TEST_WEAPON="iKhan-GPL"`: a full clip fired twice, "16" "15" "14" (take01's); `BF_TEST_CLIP=3
      BF_TEST_TRIGGER=2.5-6.5 BF_CAPTURE_FPS=30`: "3" red from the first frame and again 2.07 s later
      (take02: 2.033 s), "2" red at 4.10 s, "1" red 1.03 s on, steady red from the empty at 5.77 s,
      the hint at the release; `BF_TEST_CLIP=2 BF_TEST_TRIGGER=1.5-1.58,3.7-3.78,5.9-5.98,8.1-8.18
      BF_TEST_SWITCH=11.0`: two rounds, dry clicks (h_f740ecdd) at 6.00 and 8.20 s, no reload, and after
      the switch "iKhan-GPL" over "0" on the red panel (take02 25.0 s), the Bower 20 over "2 / 80".
    * A Seer and a hound under the player (`--test BF_TEST_GOTO=0,8,0,7.5
      BF_TEST_SQUAD_PRESET="0.5,seer coven" BF_TEST_TRIGGER=2.0-6.0 BF_TEST_RELOAD=3.0,5.0
      BF_AMMO_LOG=1`; `BF_CHARACTER=hawk` for the Fire Hound): the Psionic Blast's and the Breath
      Attack's icons over no number, never red; no reload, dry click, hint or panel line in the log.
    * Todo/56 take07 replayed (`--test BF_TEST_WEAPON="Ion Beamer" BF_TEST_CLIP=6 BF_TEST_RESERVE=20
      BF_TEST_TRIGGER=2.0-14.0 BF_CAPTURE_FPS=30`): reds every 1.03 s through 6 to 1 of 100, steady
      at 0 and through the reload (take: 4.97-6.38 s), 0.23 s after the refill to 20 (take: 0.25 s),
      a 2.03 s cycle from 20 to 11 (take: 8.48, 10.57 s) and a red at once at 10 of 100 (take: 11.76 s),
      then every 1.03 s. The shots and the reload come about 0.1-0.2 s earlier than the take's. Take08:
      the red at 1 and steady at 0 (the same rules).
    * No regressions: the ammo log's lines (reserve, reloads, dry fires, recharged rounds, hints) are
      the same as the baseline's on the Minigun (`BF_TEST_CLIP=3`, 600 → 520 at 2.47 s), the Bower
      20 (80 → 68 at the release's reload, 6.13 s) and the LZR-23 (`BF_TEST_SWITCH=1.5,4.0`, rounds
      at 1.27, 3.20, 4.47, 5.73 s); "N Regen" and "clip / reserve" read as before.
  * **Open**: the weapon box sits about 4-5 units higher in the demo than in the takes (the iKhan-GPL's
    take55 02 and the Bower's take49 02: the digits 5-6 px, the panel's border 8 px at 1.5 px a unit;
    the health bar is within 2 px), for the iKhan-GPL and the Bower alike; not traced here. The
    second row's place isn't compared either: the row classes' constructors write the words +6 = -15
    (slot 1) and 38 (slot 2), which would be 53 apart where the demo's list steps 45 (a lead, read
    as the rows' offsets in their panel, not checked against a take).
* **Automatic reload**, as in the game: on an empty clip the reload
  starts when the fire cooldown runs out with the trigger held (FUN_0022f2a0 → FUN_002327f0 →
  the character's FUN_00120d40), or when the trigger is let go (measured: Brutus's Bower tapped
  dry reloads 0.13 s after its last shot, not at its 1 s cooldown; the game's L-Shot 0.07 s after
  the release). The rounds leave the reserve at once (the takes: the reserve drops at the start),
  the clip reads 0 on the red panel until the reload clip's magazine-in event (Brutus 0.93 s,
  Tex 1.1-1.2 s; the takes 1.0-1.3 s). Not while knocked down (lying limp): it starts as they
  begin to get up. A reload drops out of the scope (FUN_00120d40 calls
  FUN_001249b0 first; medium confidence that its flag is the scope).
  * **R reloads early**, from a partial clip, once the character isn't busy. The game's X does
    it too: tapped (let go before its hold time, or with nothing to hold it for), the player
    controller (0x1332a0) calls the reload FUN_00120d40 at 0x134c9d, which starts one only when
    a reload is possible. R acts on the press, X on the release.
* **Dry fire.** An empty clip with nothing to reload clicks on each trigger cycle, once per
  1 / rate while held (FUN_002327f0 sets the cooldown): the weapon's empty-fire sound
  (`h_e6acaff7`): `f740ecdd` for the ballistic guns (spectrogram-matched in the takes),
  `16bbf95c` for the energy ones (from the data; the takes never ran an LZR dry with the trigger
  held). An empty LZR clicks too, as the code has it.
* **"Q switch to <other weapon>."** (the game's Y-button icon + "switch to %s.", `h_ff017df6`;
  the key stands for the icon, as E does in the panel prompt): once the held gun's clip and its
  reserve are both empty, from the dry click or the release, under the health bar at x 53, its
  letters at y 90-104 like the take's (`todo/49` take02 19 s). It goes with a switch away and is
  back after the switch back. Not for a recharging gun (it posts message 0x53, which shows
  nothing).
* **The weapon's name after a pickup**: the held gun's name over its count for 2.55 s, then
  fading out over 0.3 s; its icon is faint under it meanwhile (20%, judged by eye) and comes back
  as the name fades (the take: "Bower 20" from 7.0 s, fading 9.55-9.85 s). The names (also in
  the list after a switch) are the take's size: letters 13.8 units tall from y 43. A weapon
  pickup's grab shows it (take11 301.2 s "L-Shot-50 6 / 0"; #17), as the test map's rack does;
  from the pickup's release to its grab the held row's panel is empty (its icon, name and count
  gone; the faint grey shape the take has there isn't drawn).
* **The crosshair** is the held weapon's 64 × 64 texture drawn 1:1, centred at (320, 192), the
  same standing, firing and moving; in the scope it moves to (320, 240) at the same size. It's
  hidden during a weapon switch, from the old gun's drop to the new one's grab (the takes: gone
  ~0.5 s after Y, back at ~0.95 s, measured from the input to the picture; less xemu's picture
  lag and the rig's input delay, 0.14-0.21 s, the game's are about 0.33 and 0.78 s: #140,
  xemu.md "Timing in the takes"). The demo keys them on the switch clip's drop and grab events,
  which for Tex's clip fall at 0.50 and 1.13 s (`-- --test BF_TEST_SWITCH=2.0`, the log's
  "switch to weapon 1: ... drop 0.50s, grab 1.13s"; the crosshair gone and back on those frames):
  0.17 and 0.35 s later than the corrected figures, a difference to re-check from a marker-synced
  take timed by the switch's sounds before anything is changed. It stays, unchanged, while a grenade throw charges and
  through the throw: a thrown grenade has no crosshair of its own (#119; `todo/119` take01).
  Aiming and shots go through it (play.rs `CROSSHAIR_UP`). A
  weapon pickup hides it the same way, from the clip's release_weapon to its grab_weapon (take11
  300.8-301.1 s; pickups.md "Weapon pickups", #17), the test map's rack's swap too.

Checked on the test map (`BF_TEST_GOTO=0,0,0,0`, `BF_CAPTURE`), measured against the takes in
640 × 480 units:

| What | Game (take) | Demo |
|---|---|---|
| Bower crosshair box | 293.8-346.8 × 173.2-210.2 (49 take01 2.5 s) | 294.0-346.8 × 173.2-210.0 |
| LZR-23 | 27.5 × 28.2 at (320.8, 191.9) (51 take01) | 26.5 × 28.0 at (320.5, 192.0) |
| Minigun | 64.0 × 63.8 at (320.0, 191.9) (68 take01) | 63.5 × 63.5 at (320.0, 192.0) |
| Foley scoped | 33.0 × 33.2 at (319.5, 240.6) (54 take01 27.5 s) | 33.0 × 32.8 at (319.8, 240.4) |
| L-Shot, second zoom step | 8.0 × 7.8 at (320.0, 239.9) (61 take01 33 s) | 7.8 × 7.2 at (319.9, 239.9) |
| switch hint "switch" | y 90.0-104.0 (49 take02 19 s) | y 90.5-104.3 |
| name after a pickup, first letter | x 465.3, y 43.0-56.8 (49 take04 7.5 s) | x 465.8, y 43.0-57.0 |

Also checked: the Minigun's reserve 600 → 520 when its reload starts as the cooldown runs out
(held trigger, `BF_TEST_CLIP=3`), the clip full ~1.1 s later; the Bower's 80 → 68 at the
release; the hint from the first dry click; f740ecdd and 16bbf95c found in sdm_e34's banks
(`BF_SOUND_LOG`); the LZR-23 recharging held and stowed, its first round 1.7 s after the switch
away and no gap on the switch back (`BF_TEST_SWITCH=1.5,4.0`: rounds every 1.27 s through it);
a knocked-down Brutus's Bower reload held until he gets up (`BF_TEST_KNOCK`); R reloading the
Minigun at 27.40 s of `BF_MAP=sdm_e34 BF_AUTOPILOT=1 BF_START_WEAPON=1` (26 rounds).

* **A gun in the second hand** (Tex's berserk, [skills.md](../skills/texs-berserker.md)): the character's reload,
  FUN_00120d40, returns at once while the inventory holds one (+0x198), before it touches the
  reserve. So in berserk the gun in hand doesn't reload (#35 take05: 0 / 200 for 8.7 s), an empty
  one clicks dry under the trigger, each gun with its own empty sound, no "switch to" hint shows
  (FUN_00120d40 posts it), and a reload asked for (R) waits for berserk's end. Both guns' rows
  show in the weapon box, both orange.
* **Flint's targeting on** ([skills.md](../skills/flints-advanced-targeting.md)): the reload also
  asks whether the character is free (FUN_00117210, 0x120e16), which ends with the skill's
  vtable +0x14, and hers says no while it's on (0x13b980). Her empty clip and R end it first, so
  the reload starts in the frame after the off (`BF_TEST_CLIP=1`: off at 5.13 s, the reload at
  5.20). The same rule holds back a weapon switch, a grenade thrown or placed, and the medkit. The
  Wrath of Garner's skill (#208, [its page](../skills/wrath-of-garner.md)) has the same method: no
  reload for its 3 s on, one asked for (R) starting in the frame after its timer's off; from #208
  the rule also drops the crouch key and, as the demo's simplification, the X hold (a console, a
  panel, a lying gun: the game defers or cancels the completed use instead).

Test hooks: `BF_TEST_RESERVE=<n>` starts every ammo-type's reserve at n; `BF_TEST_CLIP=<n>[,<n>...]`
starts the player's clips at n (one value for every weapon, or one per weapon slot, the last for
the rest: Tex's `99,4`); `BF_TEST_TRIGGER=<from>-<to>[,...]` holds the trigger over those
windows (s; overrides `BF_TEST_FIRE`); `BF_TEST_SWITCH=<s>[,...]` presses the weapon switch once
at each time (it owns the switch for the run: Q does nothing then); `BF_TEST_RELOAD=<s>[,...]`
presses R once at each time (it owns R for the run); `BF_TEST_KNOCK=<s>` knocks the player down
then, once; `BF_TEST_CROUCH_KEY=<s>[,...]` presses Z once at each time. These run after the
test tools' input gate, so they work under `BF_TEST_FREECAM` too (the gate drops the keyboard's
and `BF_TEST_FIRE`'s input, which `read_input` takes; #227 made that order explicit after a
new system flipped it);
`BF_AMMO_LOG=1` prints the reserve, each reload, dry shot (and the second hand's), recharged
round and the hint, and (#408) each weapon slot's panel going red and going white again, with the
slot's gun and clip: "t=2.07 weapon panel goes red (weapon 0 iKhan-GPL, clip 3 of 16)". Every slot is
logged, the one not in hand too (`BF_TEST_CLIP` sets every gun's clip).
