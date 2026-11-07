*Part of [pickups/](README.md), the former page `pickups.md`.*

## Power-ups (#15)

`play_powerups.rs` (with `play_hud.rs`'s bar flash and `src/shell.wgsl`), from default.xbe's code
(the addresses below, each in [code-map.md](../code-map/README.md)) and checked against #35 take20
(sdm_e34's STAMINA POWER) and #85 take28 / take29 (sdm_e34: a barrel at 1 m with and without FORCE
POWER).

* **What they are.** BRUTE POWER, FORCE POWER, HEALTH POWER and STAMINA POWER, the glowing orbs of
  the multiplayer maps (none in the campaign): each a spawn-trigger's copy at the trigger's
  transform with its idle effect ("Pickups from spawn-triggers" above). Their item sets, by
  function-type: HEALTH and STAMINA POWER 19 IFSET_POWERUP_MEDKIT (h_0a811e94 health 50 / 0,
  h_11884f2e stamina 0 / 50), BRUTE 20 IFSET_POWERUP_BRUTAL_DAMAGE (2 and 45), FORCE 21
  IFSET_POWERUP_FORCE_SHIELD (50 and 45). sdm_e34 has one of each: BRUTE at (-0.70, 29.98,
  83.69), HEALTH (-12.30, 29.58, 77.83), STAMINA (-84.07, 29.78, 60.58), FORCE (-49.16, 29.78,
  38.85).
* **Taking one** is a walk-over (FUN_0011c540 -> FUN_0011c170), by any living member of the squad,
  the AI's too: the orb at most 1.8 m below and 1.0 m above the character's position (the hips) and
  under 1.34 m across (dx^2 + dz^2 < 1.8), as for a lying gun.
  * HEALTH and STAMINA POWER only when they can do something (their set's "can take", 0x148d40):
    the character hurt, for HEALTH; its stamina below the most, for STAMINA. Refused, the HUD of
    the player whose character it is flashes the bar it would fill (0x146900): the fill turns amber
    (the bar widget's flash colour +0x59, bytes e6 aa 3c) four times, 0.15 s each, 0.3 s apart,
    the first 0.15 s after the walk-over, and no new flash can start for 5 s more (its timer +0x48
    runs 6.2 s). take20, walked over at full stamina (2.0-4.2 s): the stamina fill amber at
    3.50-3.60, 3.80-3.90, 4.10-4.25 and 4.45-4.55 s. The amber is each texel's luminance x the
    colour x 1.64, fitted to take20's pixels (the game's colour operation isn't traced; the
    low-stamina blink's red fits the same). An AI member refused shows nothing.
  * BRUTE and FORCE always (their sets' 0x280040).
  * Taken: the item's pickup sound at the character (STAMINA h_e2824511, HEALTH h_e58a6855, BRUTE
    h_1959b73b, FORCE h_f5581a7c; FUN_00146190), PICKUP_OBJECT from the copy (its trigger's relay
    times the next one: sdm_e34's STAMINA and HEALTH POWER 30 s, BRUTE 180 s, FORCE 165 s), the
    pickup line with its name ("STAMINA POWER", 4 s: take20 9.48-13.6 s), and it's taken out of
    the world (FUN_000db3e0). Its idle effect's particles live out their lives where it was:
    take20's blue glow at Tex's feet (9.4-9.9 s).
* **HEALTH and STAMINA POWER** (0x148de0): health + 50 up to the most, or stamina + 50 up to the
  most (take20: 21.60 -> 71.68). No chatter: the "healed" line is the medkit's set's (0x146f40).
* **BRUTE and FORCE POWER** (FUN_00148b30): a modifier on the character for 45 s (h_11884f2e),
  counted down each frame, alive or dead (FUN_0011a840). Taking one of a kind that's on only puts
  its time back to 45 s (FUN_0011c170 at 0x11c458).
  * BRUTE: every damage roll the character makes x2 (h_0a811e94; FUN_0022c940's hooks, 0x15f670):
    its shots, bolts, rays and missiles, their blasts and its shots on scenery. **x2, not "quad
    damage".**
  * FORCE: every hit the character takes x0.5 (50 x 0.01), but a BLADED one (DTYPE 2) whole
    (0x125600's hooks, 0x15f7c0): shots, blasts, damage areas, gas, liquids, falls. take28: a
    radiation barrel at 1 m took 19.0 of Tex's 70 where take29's took 39.4 (x0.48).
  * Both at once multiply. `BF_COMBAT_LOG` names them: "x the shooter's power-up 2.00", "x the
    victim's power-up 0.50" (a damage area's line: "x0.50 its power-up").
  * No HUD shows the time: nothing but those hooks reads the list.
  * The "on" also sends a commentary cue (0x0f5428da BRUTE, 0xf9994826 FORCE) that every
    commentary file lists with no sound and no text: nothing plays.
* **The shell** (`ShellMaterial`, src/shell.wgsl). While a modifier lasts, its character's
  material controllers of its kind (8 for BRUTE, 0x10 for FORCE) draw the body again with the
  item's state-0 event material (`<events><event state="0" h_05db8834=..>`: BRUTE's h_1959b73b,
  FORCE's h_f5581a7c), a wrapper of a shell material (class h_0112da6f):
  * the body pushed out 0.03 m along its normals (`offset`); its guns aren't drawn so (take28's
    gun stays grey);
  * added to the picture (ONE / ONE): the texture x the colour, BRUTE's red arcs h_fab1de97 x 0.5,
    FORCE's blue arcs h_0f91fbc8 x 0.8;
  * the texture looked up by the surface's normal in view space and scrolled by the material's
    clock, BRUTE's down 0.6 a second (`translate-v` -0.6), FORCE's up 0.8;
  * the wrapper's glare key, BRUTE 120 and FORCE 150: what's brighter than that under the shell
    blooms ([levels.md](../levels/liquid-surfaces.md), "The glare").
  take28: "Tex glows blue-white" from start to end, his gun grey; so does the demo's. It shows from
  the take's frame and stays 1 s past the modifier's end (the controllers' fade-out, which they're
  drawn through). The game gives those controllers to the multiplayer squad types (mp_brutus,
  mp_hawk, mp_tex, mp_flint) and not to the campaign's brutus and hawk; the demo's squad plays the
  campaign types on every map, so it draws the shell on anyone. Each body wears an invisible
  shell for 1 s as its model first shows, so bevy has built the shell's pipeline before a take
  (without it the first shell came 1.1 s after its take).
* **Not done, or not the game's yet:**
  * (done by #286) the orbs' look: the icons' flares are GeneralApp_FacingType 2 (square to the
    particle's motion) on sphere emitters throwing them out from the middle (0.025 m/s), faded
    near edge-on (AngleFadeOut), and the demo draws them so since #286 ([maps.md](../maps/ale-blends-and-particle-facing.md),
    **Particle facing**): a glowing ball, as take20 shows at 2.0 s, where they lay flat in the
    emitter's frame before (the perp rule), a thin disc from a low camera (`BF_MAP=sdm_e34
    BF_TEST_GOTO=-79.5,63.4,-86.5,63.4 BF_TEST_GO_AT=100`, the STAMINA POWER 5.4 m off at
    (-84.07, 29.78, 60.58), take20's; scratchpad `t286dev/orb_vs_take.png`);
  * the reach against take20: the take came between 1.11 and 0.98 m across (the watch log's
    positions at 9.29 and 9.34 s) where FUN_0011c170's 1.34 m would have taken it about 0.08 s
    sooner; the refusal's flash (3.50 s, less xemu's ~0.13 s picture lag) fits 1.34 m. The code's
    reach is used; what the take measures it from isn't known;
  * the shell isn't fogged (the game's program fogs it by depth);
  * BRUTE on the grenades: their damage is the demo's own rule (#112), not FUN_0022c940's roll;
  * BRUTE's and FORCE's take and end aren't filmed: they're the code's (a shot list: BRUTE taken
    in the open with the camera on the taker, a few shots into a target, then the 45 s out to the
    shell's end; FORCE the same, with a hit taken).
  * the shell during Brutus's Vengar view: the shell is made from the body's `StandardMaterial`
    parts, which the view swaps to its heat material, so no shell shows while the view is on
    (review of #15; multiplayer power-ups and the campaign skill rarely meet);
  * a pickup line on the player's HUD when an AI squad member takes one: FUN_0011c6a0's line goes
    to the player controlling the character (to every player of the squad with AL set); whether
    an AI's take shows one isn't read (the tester of #15).
* **Test hooks:** `BF_TEST_POWERUP=<brute|force|health|stamina|h_<type>>[@<s>][;...]` gives the
  controlled character that power-up at that time (0.5 s by default) as a walk-over would, refused
  (the bar flashing) when it can't take it: the map's (or the test map's) first item of that set,
  HEALTH and STAMINA the orb that gives that. `BF_PICKUP_LOG=1` prints each power-up taken and what
  it did, each modifier's end and the shells made; `BF_COMBAT_LOG=1` the factors on each hit.
* **Verified:**
  * take20 at full stamina (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_GOTO=-81.78,63.86,-84.6,59.8
    BF_TEST_WALK=1 BF_TEST_GO_AT=0.5 BF_CAMERA_PITCH=-0.5`, 60 frames): Tex walks over it, it
    stays; the stamina fill amber at 2.33-2.40, 2.60-2.73, 2.93-3.00 and 3.20-3.33 s, as take20's
    four, the same amber against the same teal;
  * the same with `BF_TEST_STAMINA=20@0.2 BF_PICKUP_LOG=1 BF_TRIGGER_LOG=1`: taken at 2.27 s
    (stamina 72.48, +50), the orb gone, "STAMINA POWER" under Tex, the bar rising pale,
    PICKUP_OBJECT to h_0021e47f (30 s); in real time (no capture) the next copy at 32.23 s;
  * FORCE at take28's spot (`BF_MAP=sdm_e34 BF_CHARACTER=3 BF_TEST_GOTO=-53.96,24.6,-53.96,24.6
    BF_TEST_HIT=2.0 BF_TEST_POWERUP=force@1.5 BF_COMBAT_LOG=1`): Tex took 20.0 of the barrel (40.0
    without, scenery.md's take29 recreation), the shell from 1.53 s, the take's frame;
  * BRUTE on the test map (`-- --test BF_CHARACTER=3 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=0,8,0,7.3
    BF_TEST_NPC=0.3,flint,enemy,dummy,0.15,3 BF_TEST_FIRE=hip BF_TEST_POWERUP=brute@2.0
    BF_COMBAT_LOG=1`): 4.29 and 6.32 before the take, then "x the shooter's power-up 2.00" on
    every hit, 9.61-12.48;
  * BRUTE and FORCE together in the open (`BF_TEST_POWERUP="brute@0.5;force@2.0"`): the red shell,
    then red and blue adding to white; "BRUTE POWER" over "FORCE POWER".
