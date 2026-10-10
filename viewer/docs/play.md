# Playable demo (`bf_play`)

| Keyboard / mouse (no gamepad for now) | Action |
|---|---|
| WASD | move (relative to the camera); the character turns toward the direction |
| Shift | sprint |
| Ctrl | walk |
| Right mouse (hold) | aim: the camera dollies in (as when firing), the gun is turned onto the crosshair while the legs keep moving; moving away from the aim backpedals. Moving across it (30-150 degrees off the crosshair) side-steps, standing (`Sc_w1/w2_rp_f_side_walk_l/r` up to 100 degrees, `rp_b_side_walk_l/r` past it) or crouched (`cr_f/b_side_walk_l/r`). Those clips twist the upper body about a quarter turn off the root, and their root motion runs straight along it, forward or back, as far as the walks. So the character turns until the gun is on the crosshair and walks where the clip takes it: across the aim, a little forward or back. `BF_TEST_STRAFE=<x>[,<y>]` moves that way (camera-relative) with `BF_TEST_GOTO` |
| Left mouse (hold) | fire the held weapon (at its own rate; also aims while held). Standing, the feet turn with the aim (the spine keeps at most ~11° of twist, also when looking down and turning). As in the game, the camera snaps in on the first shot, holds ~0.7 s after the last, then eases back out; looking steeply down it stays out |
| Q | switch between the character's two weapons (animated: stow one, draw the other; plays the game's hard-coded switch sounds ff820fda / e1e97460, 0.65 s apart) |
| Space | jump: standing jumps crouch first, running jumps launch straight away and carry the run's momentum (Brutus uses his four-legged jump when sprinting on all fours) |
| C | dodge roll / sidestep to the left or right of the movement |
| Z | crouch / stand. Standing still: kneel (the stance's `stand2crouch`, then `rp_crouch_idle`; `crouch2stand` to get up). Moving: the crouch walk (`Sc_w1/w2_cr_walk`, `cr_back_walk` backing off while aiming; the clips' own root motion and footstep events), straight in from a run. Jump, dodge or sprint stands up. `BF_TEST_CROUCH=<s>` crouches from that time with `BF_TEST_GOTO` |
| M | cycle the ground surface (changes footstep / landing / slide sounds) |
| click, then mouse | look around (Esc releases the mouse) |
| wheel | zoom |
| 1-4 | take control of Brutus / Flint / Hawk / Tex: their name pops up in the middle of the screen in the game's own HUD font (atlas e0afcd52), with three red chevrons (the game's chevron texture 158f87c1) lighting up on the radar toward their portrait and the HUD open sound; at the cut the close sound plays and the name zooms out to 3x and fades (0.25 s, as in the capture), and the character you left drops back into the squad AI with a follow line ("I'm right behind you", "Following at a distance"...: chatter set f415ab02, from the shared voice bank `ml-sounds/en/common-en.tgz`; their speech icon shows by their portrait while they talk). Not while dead: the death camera hands over by itself |
| R | reload early: **the demo's own key** (the game has no reload button; it reloads by itself on an empty clip, see **Ammo, reload and the crosshair**). The stance's reload clip (`Sc_w1_reload` / `Sc_w2_reload`; upper body on the move, whole body standing); the clip reads 0 (panel red) until the clip's magazine-in event |
| G | use the item in the item box. A thrown grenade (Frag, Energy, Gas, Light, Sonic): hold to charge (the orange meter right of the bracket reticle, full in 0.6 s), let go to throw - the charge sets how far; the count drops at once. A placed one (Roller, Sentry): set down at the feet at the press. See **Grenades** below. The main game starts with 3 Frags |
| E (hold) | use: a gate's wall panel, from in front of it within 2.5 m, looking at it: "Hold E to activate panel." shows and a blue ring marks its button; held 0.5 s, the gate opens and stays open. See "Gates and their wall panels" |
| T | the next grenade type carried (the demo's key: the recordings don't show the game's button for it) |
| Tab | the item box (the game's B button): tap for the next item carried (each grenade type, Medkit); hold for the item list, the wheel picks one. With a Medkit selected, G heals 80 ("No need to heal" at full health). See "Health pickups" |

## Test map (`cargo run --bin bf_play -- --test`)

A flat test floor for trying things out, only reachable with the `--test` flag. It opens
straight into play: no loading screen, intro or menu (`src/bin/play_testmap.rs`).

* **Every hand weapon on a rack**, floating and turning in front of the start: the 27 weapon
  definitions with a clip whose model loads (one per model), from the first mission's data,
  the multiplayer archives (`mp_common`, `mp1`-`mp8`) and m02_a, which holds the three
  campaign-only ones (A10 Bioreactive, Confed LZR-50, Jax-iP); about 0.7 s. (m09_a adds the
  Light grenade and sdm_e34 the missile rack for the object tool: about 1 s in all.) Walk into one
  to take it into the held weapon's slot: the character is respawned carrying it, with a full
  clip, and "Took <name>" shows.
  * Loading every level (about 6 s) adds no others: the rest are the creatures' built-in
    weapons (no model), Feral Cutter (the same model as another) and the iKhan-GPL (a model
    under 0.1 m).
  * `BF_TESTMAP_LEVELS=<level>,<level>...` loads a different list.
  * `BF_TESTMAP_LOG=1` lists every weapon on the rack with its type, clip and model size, and
    every hand weapon left off it, with why.
* **One of every pickup type** with a model (one per model) in a grid behind the rack.
  * They're dropped in from 0.3 m, upright as modelled but turned at random and tipped up to
    0.2 rad, so each settles as it would. A crate lands on its base; the Garo fruit (modelled
    on its point) and the cards (on their edges) fall over onto their sides.
  * They're inventory-objects as on a map, so medkits and fruit work as in "Health pickups",
    and everything is loose (kicked, thrown by blasts, tumbling).
* **Every squad grenade type**, a full stack of each (its stack-limit, 10): Frag, Energy,
  Gas, Light, Sonic, Roller, Sentry. m09_a is
  loaded for the Light (only in m09_a/b/c/x). T steps through them. See **Grenades**.
* **Instant kill**, on at the start (`BF_TEST_INSTANT_KILL=0`: off), K toggles it (`BF_TEST_TOGGLE_KILL=<s>[,<s>...]` presses it at those times). The player's shots and grenades kill any
  squad member they hurt, in one hit. The player still takes normal damage.
* **X kills the controlled character** on the spot, as any hurt does: the death cry and
  ragdoll, then the death camera and the hand-over to the next squad member (or, with nobody
  left, the camera stays on the body). `BF_TEST_SUICIDE=<s>` presses it at that time.
* **The controls panel**, which the main game no longer shows: the keys (the tools' too) and
  the debug line (character, state, clip, surface, weapon), top left under the health bars. It
  starts hidden, with a small "H: help" at the bottom; H shows and hides it.
  `BF_TEST_HELP=<s>[;<s>...]` presses H then.
* The status message ("Took <weapon>", "Instant kill off", the tools' messages) goes after its
  2 s here too: it was counted down only on maps with doors, so on the flat floor it stayed.
* **Developer tools**: a free camera with a teleport (F), an NPC spawner (N), an object
  spawner (O, in the air from the free camera), a sky picker (Y), a music picker (U) and a level
  switch (L), which takes the test session to any level and back. On a level the rack and the
  pickup grid aren't there; the rest of this list is. `--test` with `BF_MAP=<level>` starts on
  that level. See [testtools.md](testtools.md).

## Health, HUD, locomotion, weapons and sounds

Squad: the four squad members are all in play, each with their own health (the characters'
combat-target hitpoints: Tex 115, Brutus 105, Flint 90, Hawk 65) shown on the radar's diagonal
edges. Each bar fills that diagonal's dark channel in the radar frame texture (the frame's own
texels darker than 27/255), starting by the member's portrait, and drains toward it as in the
game. For whoever you control, health also shows in the top-left bar. Grenade blasts hurt everyone in range (the explosion's Damage max, falling
off to nothing at its radius; the thrower takes a flat share, see **Grenades**) and the hurt give a pain grunt (chatter e856009f). Friendly
fire: your shots hit squadmates in the way (the weapon's damage; they say "I'm hit" or "Careful!"
/ "Stop shooting at me!"), and the crosshair turns green while it is on one. At 0 health a
character dies: death cry (chatter ef32191d), the body goes limp as a ragdoll (verlet point
masses on the skeleton, pushed by the hit or blast; its limb bones are 0.09 m spheres that don't pass through each other, and every bone lies on the level's own floors, kept as far above them as the body is thick round it (`flesh`: of the skin each bone moves most, the distance from the bone's line that 80% of it is within, 0.04-0.2 m; a flat 5 cm let a thigh or the chest sink into the floor), and is kept out of its walls - `BF_RAGDOLL_LOG=1` prints the closest limb pair and the lowest bone; the pose takes only the bones' turns from the simulation and keeps their rest offsets, so limbs don't stretch or twist; the ragdoll starts where the body is drawn, sole lift included (see "Standing on the floor"); the torso moves as two solid pieces (hips; chest with shoulders), stiffly joined at the waist, with the head held to the chest; knees and elbows (found by their Bip01 names) bend but don't fold past about 120 degrees or straighten past straight, and only one way: each joint keeps to its side of the line from the upper bone to the end, in the torso's own frame (hips for knees, chest for elbows), the side it was bent at death; a nearly straight limb uses knees forward, elbows back (forward from the toes); bodies keep 98.5% of their speed per substep, lose 60% of their sliding on the ground, and rest once still - but not on their side: a body come to rest propped on a shoulder and a hip (its chest's left-right axis more than 0.6 upright) is rolled on, its upper shoulder pushed 2 m/s toward its back (or front, if it leans that way), up to 3 times (the demo's choice). Shots hitting a body on the ground (dead or knocked down) push it: the first bone the shot passes within its thickness of is brought up to 3 m/s along the shot, and bones within 0.6 m less, so it jolts and rolls (a burst doesn't add up past that speed, so it doesn't drive the body across the ground); it wakes the body to settle again and it bleeds where it's hit, as the living do (blood effects, hit sound, a splat on the ground) (`BF_RAGDOLL_LOG` prints each body hit and the pelvis's position); `BF_TEST_DIE=<s>` drops the player dead at that time, once; `BF_TEST_DIE=1` at 1 s). Both their guns drop, the one in hand and the one on their back (the ragdoll can't feel a stowed gun, so it propped bodies on their side): each falls from where it was, thrown by half of what killed them plus a 1.5 m/s hop, and tumbles as a loose object (see "Pickups are loose"). Then the portrait greys out and the blip leaves
the radar; if you die, the death camera circles your body and control passes to the next
living member after 4 s (see **Death camera** below; `BF_TEST_KILL=<character>` makes the
player shoot that member). The inventory is shared: the
grenades go with control. Every portrait's tab shows the follow-order arrows; a speech icon
appears beside the portrait while that member talks. The one you control is the player; the other
three are AI that keep a formation on your flanks and a little ahead (running / sprinting to
catch up, walking the last bit), look where you look, and
reload on their own. They never fire just because you do - only at enemies they can see (none in the main game yet; the test map's NPCs set to fight, see [testtools.md](testtools.md)). Collision: everyone is a 0.4 m circle, pushed out of the pillars and apart from each other. Their footsteps and shots are heard quieter with distance, and they show as
yellow blips on the radar (turning with the camera, 40 m range). `BF_NO_SQUAD=1` (or `BF_DEATHMATCH=1`) plays deathmatch: alone, without the radar.

HUD health / stamina frame: the original's own art, the tutorial's picture of it (`h_17f34cbe`,
256 × 64, only in tutorial.tgz: loaded for it directly). It's cut in pieces: the + and bolt icons
(drawn solid), the frame's left cap, a middle that stretches and the right cap (drawn as glass, 45%:
Bevy blends see-through UI in linear light, so that's what matches a capture's body colour,
22 46 86, ours 17 51 91). Its baked bars are painted its own empty-bar navy (0 20 107) and the live
bars drawn over them (rows 22–27 and 35–40, from column 56). The bars' length follows the
character's maximum health (`combat-target hitpoints`: Tex 115, Brutus 105, Flint 90, Hawk 65;
`BF_DUMP_HITPOINTS=1` lists them): Tex's 184, which gives the capture's frame proportions (5.4 to
1), the others in proportion; the frame's middle stretches with them, and a squad switch resizes
it. Without the tutorial's picture, the frame below is used.

HUD textures found for the panels: the health / energy frame is built from its corner
`h_0cdf5876` (32 × 16 end cap, mirrored four ways; it holds the step from the raised ends down to
the middle, whose level its inner column carries along), drawn as glass at 55% over a faint
tint. Its two channels are open in texel rows 6–11 and (mirrored) 20–25, columns 5–174; each bar
fills its channel exactly, and its icon is level with it. The bar fills (`h_e801997b` red,
`h_e11c7d04` blue, 4 × 8) are coloured in 6 of their 8 rows, matching the channels' 6; only those
rows are drawn. (Placed by hand, the bars sat over their channels' lines and left gaps at both
ends; drawn whole, the clear rows put the colour off-centre.) The grenade's panel is
`h_10891a71` (64 × 32, glossy, cut top-left corner), 9-sliced. Neither is referenced in the
data; both were found by shape. (The tutorial's `h_17f34cbe` is only a baked picture of the
whole bar.) Radar health channels are filled as bands (distance from the centre and extent
along the diagonal, from the dark texels found), so the channels' highlight lines no longer
leave gaps.

HUD (`src/bin/play_hud.rs`), as in the game: health / energy top left, weapon panel top right
(the weapon's own icon from its definition, clip / the squad's reserve, or "clip Regen" for a
recharging gun; every weapon listed with names for 2.5 s after a switch, the held one in orange), radar with the squad's four portraits round it (Tex top, Hawk left, Flint right, Brutus bottom),
grenades bottom right, and the held weapon's own crosshair (its definition's `reticule-prefix` texture, e.g. f4ea7a92 for the MK-ASLT), 64 units square, above centre (y 192 of 480) - aiming and shots go
through it. Layout is in the game's 640 x 480 screen (measured from an xemu capture) inside a
centred 4:3 area; textures are the game's own from `common.tgz` (stored upside down, drawn
flipped). `BF_NO_HUD=1` hides it. Ammo, the switch hint and the crosshair's
size and hiding: see **Ammo, reload and the crosshair** below.

How it works (`src/bf/locomotion.rs`): animation names are hashes, so clips are chosen from
their data: root-channel velocity (forward is -Z; sprints are fastest), clean looping, head
height (rejects crouched / kneeling variants) and arm span (rejects arms-out reference poses).
The squad's only sideways clips are dodge rolls, so aimed movement is legs-run plus an
upper-body twist spread over the spine and head (`aim_chain` / `twist`). Movement is root
motion: each frame the character moves by exactly what the playing clips' root channels move,
crossfaded over 0.2 s, so speeds and footfalls match the original animations.
Jumping uses the game's named clips (`Sc_w1_jump_crouch/launch/fall/land`, `Sc_4leg_w1_jump_*`;
names recovered from the motion scripts) with simple gravity; a jump pressed during a dodge is
buffered for 0.4 s.

Weapons (`src/bf/weapon.rs`): every character carries their real starting inventory
(objecttypes: Brutus Feral Cutter + Bower 20, Flint L-Shot-50 + Confed LZR-10, Hawk Foley 356
Tact + Confed LZR-23, Tex Confed LZR-23 + RVG50 Minigun), with names from the English string
table, models from the level's weapon archetypes (24-byte static vertices; the minigun is a
compound whose barrel spins), and the definition's fire rate, fire sounds and range. A held gun
hangs from the trigger hand: the character's hand hardpoint lined up with the weapon's trigger
grip hardpoint (both point + w-x-y-z quaternion). That pairing makes every squad weapon point
straight ahead and level in the game's ready-pose clips, and puts the support hand on the
foregrip in the carry clips. Locomotion follows the weapon's stance. Each character's two motion
sets, `Sc_w1_*` and `Sc_w2_*` (idle, walk, run, dash, backpedal, reload, throw), belong to the
weapon classes of its two slots (`<limit-weapon one="CLASS n" two="CLASS n">`, class = the
weapon's `weapon-type`: 0 light, 1 medium, 2 heavy):

| Character | `Sc_w1_*` | `Sc_w2_*` |
|---|---|---|
| Brutus | medium | heavy |
| Flint | medium | light |
| Hawk | light | medium |
| Tex | heavy | heavy |

A gun takes the set of its class, else that of its slot. So Brutus holds his medium Bower 20 in
his medium stance, like the Feral Cutter, though it's his second weapon. Arms then carry each gun
the way the game animates it (Tex's minigun low in both hands, Flint's LZR-10 one-handed), and
aiming or firing switches to that set's `rp_*` ready-pose clips. The barrel is then turned onto the crosshair (yaw and pitch, two
passes, over the spine bones that carry the arm), so shots leave along the barrel (within 1
degree). The other weapon is stowed for its inventory slot: the weapon's holster hardpoint (`h_10ab40ac`,
on every gun) on the character's stow hardpoint for that slot, which puts Tex's guns vertical on
his ramps, Brutus's diagonally across his back, and Flint's and Hawk's sidearms barrel-down on
the hip. Switching plays the character's own weapon-change overlay from its motion script (Flint and
Hawk `Sc_w1_2_w2` / `Sc_w2_2_w1`, Brutus `Sc_w1_2_w2rifle` / `Sc_w2rifle_2_w1`, Tex
`Sc_wc_rifle2cannon` / `Sc_wc_cannon2rifle`, whose ramps swing the gun over his shoulder) on the
upper body over whatever the legs are doing. The clips carry timed events (an event channel of
f32 time + name hash per event): at `drop_weapon` the held gun goes to its stow point, at the
grab event the other one comes to the hand, each with the weapon's handling (`pickup-sound`)
sound; no firing or aiming until the clip ends. Shots: muzzle flash and light,
recoil, tracers that fly to what the crosshair is on (ground or pillars) and a spark there.

### Ammo, reload and the crosshair (#114)

`src/bin/play_ammo.rs`, the HUD parts in `play_hud.rs`. Logic from `default.xbe` (functions cited
in the code), checked against the weapon takes in `todo/49-*` … `todo/68-*`.

* **The reserve is the squad's**, one count per ammo-type, shared by every member and every gun
  of that type (FUN_0022da80 looks the weapon's `ammo-type` up in the holder's inventory). The
  Foley 356 and the L-Shot share 11mm Ammo (type 1); the MK-ASLT and the Minigun High ROF Ammo
  (type 2). The panel reads "clip / reserve" (orange), e.g. "24 / 50" for Hawk's Foley.
  * Each level starts with a full stack of every type: the stack-limit of the level's ammo item
    (objecttypes `<inventory>` with function-type 14, IFSET_AMMO_BOX; its h_e5f51266 is the
    ammo-type): 11mm 50, High ROF 600, Sonic 200, Particle 200, Rail 400, Shotgun 80, Cutter
    120, Bio 200, Rocket 40, Energy 200. The same in every level that defines them, so a level
    without the item uses those. The game's counts are the campaign's (the takes start with full
    11mm, Sonic and Shotgun, half the High ROF and Cutter); the full stack is the demo's choice.
  * A respawn (the test map's rack) keeps the reserve; the new gun comes with a full clip. The
    test map's NPCs reload from a bottomless reserve of their own.
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
* **Automatic reload**, as in the game, which has no reload button: on an empty clip the reload
  starts when the fire cooldown runs out with the trigger held (FUN_0022f2a0 → FUN_002327f0 →
  the character's FUN_00120d40), or when the trigger is let go (measured: Brutus's Bower tapped
  dry reloads 0.13 s after its last shot, not at its 1 s cooldown; the game's L-Shot 0.07 s after
  the release). The rounds leave the reserve at once (the takes: the reserve drops at the start),
  the clip reads 0 on the red panel until the reload clip's magazine-in event (Brutus 0.93 s,
  Tex 1.1-1.2 s; the takes 1.0-1.3 s). Not while knocked down (lying limp): it starts as they
  begin to get up. A reload drops out of the scope (FUN_00120d40 calls
  FUN_001249b0 first; medium confidence that its flag is the scope).
  * **R reloads early: the demo's own key** (the product owner's choice), from a partial clip,
    once the character isn't busy.
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
  the list after a switch) are the take's size: letters 13.8 units tall from y 43.
* **The crosshair** is the held weapon's 64 × 64 texture drawn 1:1, centred at (320, 192), the
  same standing, firing and moving; in the scope it moves to (320, 240) at the same size. It's
  hidden during a weapon switch, from the old gun's drop to the new one's grab (the takes: gone
  ~0.5 s after Y, back at ~0.95 s). Aiming and shots go through it (play.rs `CROSSHAIR_UP`). The
  game also hides it during its pickup motion; the test map's rack has none.

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

Test hooks: `BF_TEST_RESERVE=<n>` starts every ammo-type's reserve at n; `BF_TEST_CLIP=<n>`
starts the player's clips at n; `BF_TEST_TRIGGER=<from>-<to>[,...]` holds the trigger over those
windows (s; overrides `BF_TEST_FIRE`); `BF_TEST_SWITCH=<s>[,...]` presses the weapon switch once
at each time (it owns the switch for the run: Q does nothing then); `BF_TEST_KNOCK=<s>` knocks
the player down then, once;
`BF_AMMO_LOG=1` prints the reserve, each reload, dry shot, recharged round and the hint.

### Shots: fire timing, pellets, accuracy, holes, muzzle effects, tracers, casings (#115)

`src/bin/play_shots.rs` follows default.xbe's weapon class (0x22b000-0x233000); the attributes it
reads are `weapon::ShotData` (each with its data offset and the function that reads it).

* **Fire timing** (FUN_0022f2a0, FUN_0022f0e0, FUN_0022ec90). The fire loop runs in 30 Hz game
  frames. A shot *sets* the cooldown to 1 / rate (the rest of the frame is dropped), it counts
  down a frame at a time, and the trigger fires again once it's <= 0. So an interval is the
  rate's period rounded up to whole frames: MK-ASLT 8/s -> 4 frames, 7.5/s (the takes: 7.0-7.4,
  the emulator ran at 27-29 fps). Most periods are a whole number of frames (Minigun 2, Foley 10,
  LZR-50 8, Jax-iC 15): the cooldown lands on 0 and the frame time's wobble decides between
  that frame and the next, about half each, which the takes show (Minigun 0.086 s a round =
  2.5 frames, ~12/s; Foley 0.352 = 10.5; LZR-50 0.284 = 8.5). The demo models that wobble as
  +-0.01% on each 30 Hz frame (`TICK_JITTER`), which moves no other period. In
  the scope the rate is h_019c314a (the MK's 6/s; medium confidence that the holder's +0x7a8 is
  the scope). The demo's 0.2 s raise before the first shot from idle is unchanged.
* **Bursts and pellets**. A trigger cycle is h_e4076713 shots, h_e6c60892 s apart. At 0 s apart
  (the Bower 20's 6) they all go in the same frame for one round and one report, and each after
  the first is turned from the one before by up to h_e704fd69 degrees of yaw and of pitch
  (FUN_0022e1d0: the pellets walk away from the first; medium confidence on the walk).
  bullet-type 4 is an instant ray whatever the bullet's speed (FUN_0022f1d0): the Bower's and
  the MK's 80 m/s aren't used.
* **Accuracy** (weapon+0x1dc). The accuracy block h_fb327295 {min, max, scoped cap,
  per shot, per second}: the current accuracy A is held in [min, cap] (cap: max, or the scoped
  cap in the scope), each trigger cycle takes "per shot" off, and each frame the cooldown is out
  it regains "per second" x dt (crouched: half the loss, a floor of min + 15, twice the gain;
  FUN_00222ac0 / 00222c10 / 00222b30). **Where it's used** (found for this ticket):
  FUN_0022e4c0, once a frame before the fire loop, turns each muzzle's aim about the three world
  axes by uniform random angles within +-S degrees (FUN_002229b0), with
  S = (100 - clamp(A - 1, 0, 100)) x 0.03 (FUN_000c4e10; the -1 is the holder's script bonus,
  unset). So the MK's first shot spreads +-0.33 degrees and its 14th on in held fire +-2.3; the
  Minigun's +-1.2 to +-1.8 over a clip; the Bower's aim +-0.5 before its pellets' walk.
* **Damage falloff** (FUN_00230040 -> FUN_00223780). An instant hit's damage is scaled by the
  Damage's h_fb124e6c mode at x = 1 - distance / range: 1 x, 2 x^2, 3 full to half range then
  2x, else none. Only the Bower has one (2): 0.60 at 9 m of its 40 m range. It's applied to the
  squad's, the NPCs' and breakable scenery's hits. (The rest of the take's ~1.3x on the
  trooper isn't found yet: see the ticket's report.)
* **Holes** (`play_fx.rs`, `HoleRequests`). Every gun shot that meets the world leaves its
  bullet's h_06a27365 decal (0.3 x 0.3 m holes and scorches, read as full sizes; 25 s, fading
  over the last 5) laid on the surface it met: the map's collision triangle
  (`Arena::ray_normal`) or, on the flat test floor, the ground or a pillar's side. Up to 160 stay
  (a guess).
* **Muzzle effects**. The weapon's h_fd88830d effect plays at its muzzle hardpoint once a frame
  in which it fired (the MK's orange flash and grey smoke); it replaces the plain flash quad on
  guns that have one. The point light stays (the demo's). In the controlled character's own
  scope it's drawn out along the barrel 3 m from the eye instead (`SCOPE_FLASH_REACH`, the
  demo's rule): at the muzzle, a few tenths of a metre from the scoped eye and magnified, its
  sparks covered half the view. The game's scope shows a modest orange ball right of the
  crosshair (63/take01, 28.6 s); the scope-in code (0x1223da, FUN_001249b0) doesn't move or
  hide it, and nothing else that does was found. At 3 m it is about that size and place; its
  look is still the demo's ALE streaks (a star), not the game's soft ball.
* **Tracers** (FUN_002317e0). An instant ray carries its flight effect only when its counter is
  0, which then restarts at h_eeb9e75a: the MK every 3rd shot, the Minigun every 4th, the rest
  every shot. The tracer is now turned along the shot (ALE emitters fire along their +y): it
  flew straight up before, the thin vertical line over the Minigun.
* **Casings** (FUN_0022eee0 -> FUN_00231b10). Each trigger cycle owes h_fdc93b33 casings; each
  whole one throws the h_19b21bf5 object (the casing h_16415157's model) from the h_ea1abbbe
  hardpoint, 5 cm back along its z, at 2-3 m/s along its z turned a quarter turn about the
  vertical, plus up to 0.5 m/s on each other axis. MK, Minigun, L-Shot and Foley; the Bower has
  the hardpoint but no casing object. Their 2 s life, gravity and bounce are guesses.

Test hooks: `BF_TEST_WEAPON=<label or hex>` (e.g. `"Bower 20"`, `"RVG50 Minigun"`, `0c3db625`)
puts that weapon in the controlled character's slot `BF_START_WEAPON` (default the first) if the
map's data has it (the test map has every hand weapon); `BF_SHOT_LOG=1` now also prints each
pellet (its frame, spread, walk, tracer) and the gun's accuracy; `BF_TEST_FIRE=hip` fires
without aiming (no scope on a gun that zooms).

Verified on the test map (`-- --test`, `BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12
BF_TEST_FIRE=1 BF_SHOT_LOG=1`, the pillar 10 m ahead) and sdm_e34:
* Minigun (`BF_START_WEAPON=1`): 2-3 frames a round, 12.3/s over 66 rounds; a tracer on every
  4th; accuracy 59 -> 10 over the clip. Seen from 13 m (`BF_TEST_FREECAM=4.4,1.5,-0.2,-4.7,..`)
  its holes span ~+-10 units up and down, the spec's cluster (+-10).
* MK-ASLT (`BF_TEST_WEAPON=MK-ASLT`): 4 frames, 7.5/s; a tracer on every 3rd; scoped 5-6
  frames (5.6/s). The game's MK holes after the take (63_zoom) spread ~+-26 x +-18 units, about
  the +-2.3 degrees the code reaches in held fire.
* Bower 20 (`BF_CHARACTER=0 BF_START_WEAPON=1`): one round and one report per shell, 6 pellets
  walking up to ~15 degrees from the first; holes on the pillar and on sdm_e34's walls; its
  muzzle effect (white flash, orange ball, smoke) along the barrel in a side view
  (`BF_CAMERA_DISTANCE=2.5 BF_VIEW_YAW=1.3`).
* Casings land to Tex's right on the floor; the muzzle smoke and flash at the Minigun's barrel.
* The main game on sdm_e34 (`BF_AUTOPILOT=1`, 340 frames) is pixel for pixel the same as
  before until the autopilot's first shot.

Sounds (`src/bf/audio.rs`) come straight from the game's banks: `sounds-<level>.xmb` maps sound
ids to Xbox ADPCM data in `sounds-<level>.mem`, decoded to 22 kHz WAV in memory. Each character's
archetype names its jump grunt and footstep type (Tex 1, Flint 2, Brutus 3, Hawk 4), and
`world-materials.xmb` gives every surface four footstep variants per type plus landing and slide
sounds. Footsteps fire when a toe comes down into the lowest quarter of its swing in the clip
being played. Surface sounds live in level archives: `BF_LEVEL=<name>` (default `e01`) picks the
level; `mp1`'s bank is merged in too, since a mission only carries its own squad's footsteps.

Test hooks: `BF_AUTOPILOT=1` plays a scripted run (idle, run, turn, sprint, aimed walk,
backpedal, dodge, standing jump, running jump, aimed fire, weapon switch, firing on the run); `BF_TEST_EXPLOSION=1` sets off the selected grenade type 6 m ahead every 1.5 s; with `BF_TEST_GOTO`, `BF_TEST_THROW=<s>[,<hold s>,...]` holds the grenade button (G) at those times (see **Grenades** for the rest of its hooks); `BF_ANIM_PROBE=<script>,...` prints each character's clips by script name (duration, root motion over a cycle, events); `BF_DUMP_TEXTURE=<file>:<hex id>` writes a decoded texture (raw RGBA after a u32 width and height). `BF_AUTOPILOT_SPIN=<rad/s>` makes it look down and turn the camera during the standing fire; with `BF_CAPTURE=<dir>` it saves every frame at a
fixed 15 fps. `BF_MUTE=1` silences audio, `BF_SOUND_LOG=1` prints every sound event with its time
and action, `BF_DUMP_SOUND_IDS=<dir>:<id,id..|all>` decodes sound ids to WAV, `BF_DUMP_SOUNDS=<dir>` writes the character's jump and surface sounds as WAV files
and exits. `BF_DUMP_WEAPONS=1` lists every character's weapons as loaded (definition, model parts,
hardpoints, fire sounds) and exits; `BF_SHOT_LOG=1` prints each shot's barrel and shot
directions. Camera hooks: `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` / `BF_CAMERA_DISTANCE` set the
start (with `BF_TEST_GOTO`, `BF_CAMERA_PITCH` holds the pitch throughout, e.g. to fire down at
something), `BF_VIEW_YAW` turns only the rendered view (aim unchanged), `BF_START_WEAPON=1` starts with
the second weapon. Captures wait 60 frames for shaders to compile before the clock starts.
