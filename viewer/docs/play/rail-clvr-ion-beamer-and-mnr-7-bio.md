*Part of [play/](README.md), the former page `play.md`.*

### The Rail CLVR (#66), the Ion Beamer (#56) and the MNR-7 Bio (#64) against #287's takes

Nothing of their own to build: each fires through the shared machinery from its own data, and
was run on the test map (`-- --test BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=4.5,20,4.5,20
BF_TEST_NPC_HP=1.0,all,300 BF_SHOT_LOG=1 BF_COMBAT_LOG=1`, dm Shrike Soldiers frozen on the
line). Each is compared below with #287's takes (`todo/287-weapon-shots/`, their specs in
todo/56-ion-beamer/ and todo/64-mnr7-bio/): the Rail CLVR's 14-17, the Ion Beamer's I1 shot
(take22, Brutus at m09_a's bench) and the MNR-7's 18-21 (Hawk at the same bench). Their
breakpoint takes (19, 21, 23) read the order and the amounts of the hits, the clean ones (18,
20, 22) the timing: each logged hit stalls the VM.

**Rail CLVR** (#66; h_1b0310e6, heavy: Tex): the Rail RVR's ray and beam (§ Ricochet and
pass-through, § Tracers and beams: its flight effect h_fa3b46ce plays the same three, rgun_perp_s,
rgun_side_s and rgun_beam; its hit h_f79e1c48) with its own data: 3 a second, clip 30,
11.5-15.5 BALLISTIC with no falloff, n1 1 and n2 500. Against #287's takes on m14_a, Tex at the
start ledge (take14's spot):
* Held fire one round every 0.33-0.40 s, 0.35 on average (`-- --test BF_MAP=m14_a BF_CHARACTER=3
  BF_TEST_WEAPON="Rail CLVR" BF_TEST_GOTO=-106.0,-173.0,-106.0,-166.0 BF_TEST_GO_AT=100
  BF_CAMERA_PITCH=-0.10 BF_TEST_TRIGGER=3.0-3.08,5.4-5.48,7.8-7.88,10.5-15.5 BF_SHOT_LOG=1`;
  these runs had Tex at m14_a's squad spawn, (-104.3, -163.1), which moved him off the GOTO's
  start before #292; the start is kept since, 10 m from there, so the Shrike below is 7 m out
  where it was 3 m; the Rail's damage has no falloff and its timing no place): take14's held
  fire (clip 26 -> 17 at game times 10.07-13.16) one every 0.339-0.346 s, 0.343 on average.
* Damage on a frozen Shrike Soldier 7 m out (take16; the same with `BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC="0.3,shrike soldier@m14_a,enemy,frozen,-106.0,-166.0" BF_TEST_NPC_HP=1.0,all,300
  BF_COMBAT_LOG=1`): 16.1-21.0 on the body (the roll x 1.4 on a x1 part; 12.9 on an upper arm),
  take16's 16.98-20.52 on the body and 99.9 on the head (x5) of its first shot.
* Pass-through (#287 take17: the Shrike 10 m out, then the object 7f20f200 behind it twice): the
  RVR's rule, settled against takes 01-03 (§ Ricochet and pass-through, "Against #287's takes").
  Here the ray goes through the Shrike, meets the ground 14 m out (walls 0) and again 35 m out
  (-1), and stops.
* Reload (`-- --test BF_NO_SQUAD=1 BF_CHARACTER=3 BF_TEST_WEAPON="Rail CLVR" BF_TEST_GOTO=0,-8,0,-8
  BF_TEST_CLIP=3 BF_TEST_TRIGGER=2.5-4.0 BF_AMMO_LOG=1 BF_SOUND_LOG=1`): rounds at 2.73, 3.07
  and 3.40, the reserve 400 -> 370 0.33 s after the last, and the clip full at the reload clip's
  magazine-in (its sound) 1.00 s after that. take15's watch log: the rounds at 2.388 / 2.683 /
  3.076, the reserve at 3.420 (+0.344), the clip 30 at 4.652 (+1.232). The magazine-in comes
  0.23 s early: Tex's reload clip, as #63's MK found it (Tex's takes 1.1 and 1.2 s against the
  demo's 1.00; Hawk's 1.0-1.1 match), not this gun's (found, not in scope).
* Its beam: the RVR's, open (§ Tracers and beams, "Not matched"). In take14 it's a thin violet
  line from Tex's head up to the reticle, with a few faint rings: its near part behind his head.
* Not checked again (generic): the dry fire and the switch (take15: two clicks with the reserve
  at 0, then Y to the Saryl-99 and back).

**Ion Beamer** (#56; h_ea241276, medium: Flint, `BF_CHARACTER=1 BF_TEST_WEAPON="Ion Beamer"
BF_CAMERA_PITCH=-0.2 BF_TEST_NPC="0.3,shrike soldier@dm,enemy,frozen,4.5,6;0.3,shrike
soldier@dm,enemy,frozen,4.5,1" BF_TEST_PLACE="0.5,supply crate,4.5,16,0;0.5,supply crate,4.5,12,0"
BF_TEST_TRIGGER=2.0-2.08,3.5-6.0`): a bolt at 100 m/s (Flint's rifle range x1.25: 112.5 m) with
the pass flag and n1 5, n2 0 (#238): it goes through both crates (each dealt the launch roll,
its violet hit at each) and stops at the first Shrike (16.2 on an upper arm, the roll x 0.8 x
1.4), never the one behind; clip 100, 2 a second (the cycle below).
"Penetrates armour" is its damage-type 6 (PARTICLE) against each target type's factor (generic,
`play_damage.rs`); "some types of walls" are the objects the code lets it through (a placed
object, a gate, a crate, a broken object's remains: the ray result's kind 2), never the terrain,
which ends it. Against #287 take22 (#56), at its spot on m09_a (`-- --test BF_MAP=m09_a
BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_WEAPON="Ion Beamer" BF_TEST_GOTO=-181.8,-221.4,-167.0,-221.3
BF_TEST_GO_AT=100 BF_TEST_FREECAM="0.3,-181.8,76.6,-219.78,0,-89.9" BF_TEST_TELEPORT=0.5
BF_TEST_FREECAM_OFF=0.7 BF_TEST_NPC="0.9,shrike soldier@m09_a,enemy,frozen,-167.5,-221.3,66"
BF_TEST_NPC_HP=1.5,1,300 BF_TEST_INSTANT_KILL=0 BF_CAMERA_PITCH=-0.19
BF_TEST_TRIGGER=3.0-3.25,5.5-5.75,8.0-8.25 BF_SHOT_LOG=1 BF_COMBAT_LOG=1 BF_CAPTURE_FPS=60`; the
Shrike spawned from 66 m up, as at 61-62 the floor search under it misses the bench and finds one
at -1):
* The first bolt passes the bench's spike (breakable 183, take22's 7f1f6600) 6.1 m out on its
  first sweep, 0.1 s after the shot (n1 5 -> 4), and strikes the Shrike 12.8 m out on its second,
  0.2 s: take22's HP readings, the spike's then the Shrike's 0.09 s apart on every shot (FUN_00225440's
  sweep each third game frame, 10 m a sweep at 100 m/s). One roll serves both: the demo's first,
  14.44, is take22's first, the Shrike's 20.2 its 20.21 (x 1.4); the next 15.04 and 14.87 give
  21.1 and 20.8 (take22: 16.59 -> 23.24, 16.34 x 5 -> 114.35 on the head, 16.74 -> 23.44, 14.57 ->
  20.39). Nothing is dealt less for the pass (no attenuation on that path, take22).
* The look (frames 192-221 at 60 fps; take22 f004-f034 of 3.95 s on): a small violet flash on
  the spike at +0.1 s, then a blue-white glow at the Shrike from +0.2 s with fine white arcs
  forking off it, ~0.3 s (take22's bright blue-white pixels by the reticle: on at 4.20-4.30 s, off
  at 4.567; the demo's over frames 204-222). No streak in flight in either. The demo's glow is
  larger and softer (its lit pixels peak ~1.5 times take22's).
* The fire sound h_1733d6e2 with each shot (`BF_SOUND_LOG`); its HUD icon h_e9e20036 (128 x 64)
  and dashed-circle crosshair h_17e53364 as take22's.
* Against #56's takes 01-08 (todo/56-ion-beamer/, its spec; m09_a's bench, Brutus):
  * Where the bolt ends: take01's hit log (take22's staging) has each bolt hit the spike, then
    the Shrike, and nothing after (not the rock 1.5 m behind); takes 02-04 a second Shrike 4 m
    and 1.2 m behind the first or 0.5 m beside the line, never hit; take05 the rock face 16 m out,
    a kind 1 hit that ends the bolt. The demo's rule (n2 0 at the first character, the terrain
    ends it: § Ricochet and pass-through) is the same.
  * Held fire (take06): one round every 0.503 s over 7 rounds, take07's 0.511-0.513 s over 18
    (the game's clock; the 20 Hz log). The demo's cycle doesn't round 1 / rate up: the cooldown
    runs down by the frame time and 0.5 s lands on the 15th frame, where the frame clock's
    wobble (play_shots.rs `TICK_JITTER`) gives 15 or 16 frames about half the time each (with no
    wobble the f32 countdown is left 2e-8 above 0 after 15 frames: 16 every time). Held 22 s on the test
    map (`-- --test BF_NO_SQUAD=1 BF_CHARACTER=0 BF_TEST_WEAPON="Ion Beamer"
    BF_TEST_GOTO=0,-8,0,-8 BF_TEST_TRIGGER=2.0-24.0 BF_SHOT_LOG=1`, the pellets' ticks): 17
    intervals of 15 frames and 25 of 16, 0.520 s on average, 3 % over take06-07's 0.51 (#35
    take04's Jax-iC, the same 2 a second, measured 0.52; the other guns whose period is a whole
    number of frames, the Minigun, Foley, LZR-50, are the same rule, TICK_JITTER). The first
    round 0.17 s after the trigger (take06: ~0.25 s after the pad press, the controller's poll
    in it).
  * Reload (take07, a clip of 6 and 20 in reserve, the trigger held; the demo's
    `BF_TEST_CLIP=6 BF_TEST_RESERVE=20 BF_TEST_TRIGGER=2.0-9.0 BF_AMMO_LOG=1`): the reserve taken
    0.53 s after the last round (take07: 0.512 s, the cycle's cooldown), all 20 of it into the
    clip, and the trigger still held, the next round 1.37 s after that (take07: the clip full
    0.937 s in, the next round 0.485 s after: 1.42 s).
  * Dry fire, the panel's red "0 / 0", the "(Y) switch to Confed LZR-50." hint from the clip's
    last round and the switch (take08): the generic rule (§ Ammo), checked on the MNR-7.
  * Open: the demo's spike breaks on the first bolt (breakable 183's own hit points), where
    take22's trainer raised the spike's to 1000, so the later bolts find it gone (no hook sets
    an object's hit points). The panel's low-ammo blink (takes 07 and 08: red for 0.23-0.25 s
    every ~1.05 s while the clip is at 16 or less of 100) isn't in the demo (found, not in
    scope). Not filmed: a level wall or gate leaf (the Ion Beamer is only in m04 / m09 data, and
    m09_a's bench has neither), the bolt and its hits in daylight or from the side, the flight
    voices h_e297c1ef / h_f099a0c3 (the bolt's in-flight voice isn't played, answers-batch1 §3).

**MNR-7 Bio** (#64; h_e671e0f8, light: Hawk, `BF_CHARACTER=2 BF_TEST_WEAPON="MNR-7 Bio"
BF_CAMERA_PITCH=-0.16 BF_TEST_NPC="0.3,shrike soldier@dm,enemy,frozen,4.5,10"
BF_TEST_TRIGGER=2.0-2.08,3.5-6.0`): each trigger cycle fires three darts at once (its burst of
3, 0 s apart, walked by the burst's 3.5 degrees: #115's pellets; the log's three ways lie 3-4
degrees apart) for one round off the clip (50 -> 49 -> 48: FUN_0022fe00), one cycle every 15 or
16 frames (2 a second: 0.52 s on average, the Ion Beamer's cycle above); each dart is a bolt at 80 m/s with its own roll (10-12 BIO) and its 40 m range (a
0.5 s life), and lands 0.2 s after the shot 9.6 m out (14.0-16.0 on a x1 part; 81.5 on the
head); one to three of them hit the Shrike, the walked ones passing it. The green darts
(bio_bullet_pistol and its trail) and the green bio flash where they strike (bio-flash_hit,
bio-shrap_hit, light_hit_bio). Against #287's takes 18-21 (#64), at take18's rock face on m09_a
(`-- --test BF_MAP=m09_a BF_CHARACTER=2 BF_TEST_WEAPON="MNR-7 Bio"
BF_TEST_GOTO=-187.35,-219.9,-187.35,-229.2 BF_TEST_GO_AT=100
BF_TEST_FREECAM="0.3,-187.35,76,-218.28,0,-89.9" BF_TEST_TELEPORT=0.5 BF_TEST_FREECAM_OFF=0.7
BF_CAMERA_PITCH=0.0 BF_TEST_TRIGGER=3.0-3.25,5.4-5.65,7.8-8.05,10.5-14.0 BF_SHOT_LOG=1
BF_CAPTURE_FPS=30`; the free camera 16 m over the bench, so the teleport's 1.62 m offset holds):
* **The impacts' spread is the sweeps'**, settling the spec's open question. FUN_0022f2a0's loop
  fires all three darts in one frame (the burst delay 0 sets a cooldown of 0, so it goes round
  again in the same call; FUN_0022fe00 excuses the first two their round), FUN_0022bac0 gives
  each the bullet's speed with nothing added (+0x1c from the weapon's +0x210; h_ea5327e6 isn't
  read there), and every pooled bolt's sweep counter starts at 3 (FUN_00225000 sets byte +3 when
  the pool hands it out; FUN_00225440 counts it down): no per-dart phase. A dart's first sweep
  reaches 8.0 m from the muzzle at 80 m/s, and take18's rock face is 7.4-8.2 m from it, so a dart
  walked up or out past 8.0 m lands on the second sweep, 0.1 s after the others: take18's two
  lobes 0.1 s apart in most cycles, one lobe (all three together) in its cycle 4; its third
  flash at +0.13 s is the second-sweep pair's flash growing over two pictures (536 then 1366 white
  pixels). take19's hit log agrees: tap 1's darts at 8.05, 8.01 and 8.20 m from the weapon object,
  the last logged last. The demo, the same: all three launched at once (t=3.17), bolts 1 and 2 at
  7.7 and 8.0 m on frame 3, bolt 3 at 8.1 m on frame 6, and two such split cycles in ten (one
  dart of the second tap met Tex, whom the recipe leaves in the squad, walking through the line).
  The breakpoint logs' 0.06-0.13 s between darts are their stalls.
* Held fire one cycle every 0.47-0.54 s for one round (take18 and take20: 0.47-0.54); the clip
  50 -> 49 on each tap.
* Reload (`-- --test BF_NO_SQUAD=1 BF_CHARACTER=2 BF_TEST_WEAPON="MNR-7 Bio" BF_TEST_GOTO=0,-8,0,-8
  BF_TEST_CLIP=6 BF_TEST_TRIGGER=2.5-5.5 BF_AMMO_LOG=1 BF_SOUND_LOG=1`, take20's 3 s hold): rounds
  at 2.73-5.33, the reserve 200 -> 150 at the release, 5.60 (0.27 s after the last round), and the
  clip full at the magazine-in (h_08081139) 1.00 s later. take20 held its trigger to 5.507 s of
  video, its game time ~5.12: the reserve fell at 5.137, 0.196 s after the last round, at that
  release (FUN_00121170's action 4), and the clip read 50 0.976 s after it. Held on past the
  empty, the demo's reload waits for the empty cycle at the cooldown's end (0.54 s), as the other
  guns' do.
* Dry fire (`BF_TEST_CLIP=2 BF_TEST_RESERVE=0 BF_TEST_TRIGGER=3.0-3.25,5.4-5.65,7.8-8.05,10.2-10.45`):
  two rounds, then a click (h_16bbf95c) 0.07-0.13 s after each press (take20 0.07-0.095 s), no
  reload; the switch hint up at the release after the last round (take20: at the empty, before the
  first click).
* Damage per dart on a frozen Shrike, the roll x 1.4 on a x1 part, 14.0-16.0 (take21: 14.20-16.76,
  one 7.83 on a x0.5 part, BIOREACTIVE x1 on this Shrike).
* **Its HUD icon at its own size** (`WeaponDef::icon_size`, play_hud.rs `WEAPON_ICON`): its item
  names a 64 x 64 icon (h_e2bc2c81 x h_f71189bb, FUN_001920d0's +0x19c / +0x1a0), and take18 draws
  it so at the panel's left, where the demo stretched every icon over 128 x 64. The same for the
  other pistols, the Confed LZR-10, the Foley 356 Tact, the Saryl-23 and the Jax-iP (#54 take01's
  Foley icon is 64 wide too).
* Open: the switch hint's lifetime: take20's showed 3.72 s from the empty (gone before its second
  dry click) and 1.90 s after a switch back, where the demo's stays while the gun is dry; the game
  posts CBT_OUT_OF_AMMO (FUN_00120d40, while +0x7e8 is clear) and the HUD's line for it isn't
  traced (a code question). The darts in flight: the demo's three trails (bio_bullet_pistol_trail,
  0.44 m cards for 0.175 s) make a broad green haze from the gun toward the wall, where take18
  shows small green orbs (6-9 px) with faint short trails; the rule is the A10's checked one
  (#47), so it's left. The switch, generic. Blocked: the bioreactive kill (#147).
