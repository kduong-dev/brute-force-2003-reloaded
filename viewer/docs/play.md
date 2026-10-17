# Playable demo (`bf_play`)

| Keyboard / mouse (no gamepad for now) | Action |
|---|---|
| WASD | move (relative to the camera); the character turns toward the direction |
| Shift | sprint (the game's dash). `BF_TEST_SPRINT=<s>` sprints toward the goal from that time with `BF_TEST_GOTO` |
| Ctrl | walk |
| Right mouse (hold) | aim: the camera dollies in (as when firing), the gun is turned onto the crosshair while the legs keep moving; moving away from the aim backpedals. Moving across it (30-150 degrees off the crosshair) side-steps, standing (`Sc_w1/w2_rp_f_side_walk_l/r` up to 100 degrees, `rp_b_side_walk_l/r` past it) or crouched (`cr_f/b_side_walk_l/r`). Those clips twist the upper body about a quarter turn off the root, and their root motion runs straight along it, forward or back, as far as the walks. So the character turns until the gun is on the crosshair and walks where the clip takes it: across the aim, a little forward or back. One clip in the squad's data has no root motion: Tex's `Sc_w1_rp_f_side_walk_l` (his first slot, whatever gun is in it, aimed left; its own speed h_f9628571 is 5.9e-05 m/s where the squad's other side steps carry 1.0-2.1). The game doesn't move a walk by its clip (FUN_0012c6c0 moves it at the type's `speeds` along the stick), so there he steps left as he does right. The demo moves by root motion, so a side step whose own speed is under 0.001 (the game's no-speed threshold for the play rate) takes the root motion of its stance's walk in the same direction, forward or back (`root_stand_ins`; #170: he walked in place before). `BF_TEST_STRAFE=<x>[,<y>]` moves that way (camera-relative) with `BF_TEST_GOTO`; with a goal far ahead (`BF_TEST_GOTO=4.5,20,4.5,-80`) the camera keeps still while he steps. `BF_MOVE_LOG=1` prints the controlled character's position, facing, gait and clip every half second (and the stand-in walk, `root from`). Verified (#170, `BF_MOVE_LOG=1 BF_TEST_AIM=1 BF_TEST_GOTO=4.5,20,4.5,-80 -- --test`, Tex with the Therm Sweeper): aimed left (`BF_TEST_STRAFE=-1,0`) 1.59 m/s and right 1.60, back-left / back-right 1.53 / 1.54, crouched 1.72-1.73 each way; the planted boot holds within 1 px of the floor on both steps; main's build stepped left in place; Hawk's aimed steps and Tex's unaimed walk are pixel-identical to main |
| Left mouse (hold) | fire the held weapon (at its own rate; also aims while held). Standing, the feet turn with the aim (the spine keeps at most ~11° of twist, also when looking down and turning). As in the game, the camera snaps in on the first shot, holds ~0.7 s after the last, then eases back out; looking steeply down it stays out |
| Q | switch between the character's two weapons (animated: stow one, draw the other; plays the game's hard-coded switch sounds ff820fda / e1e97460, 0.65 s apart) |
| Space | jump: standing jumps crouch first, running jumps launch straight away and carry the run's momentum (Brutus uses his four-legged jump when sprinting on all fours) |
| C | dodge roll / sidestep to the left or right of the movement |
| Z | crouch / stand. Standing still: kneel (the stance's `stand2crouch`, then `rp_crouch_idle`; `crouch2stand` to get up). Moving: the crouch walk (`Sc_w1/w2_cr_walk`, `cr_back_walk` backing off while aiming; the clips' own root motion and footstep events), straight in from a run. Jump, dodge or sprint stands up. `BF_TEST_CROUCH=<s>` crouches from that time with `BF_TEST_GOTO` (held: asked for again each frame); `BF_TEST_CROUCH_KEY=<s>[,...]` presses Z once at each time, as the key toggles. In Tex's berserk Z does nothing and the press is dropped ([skills.md](skills.md)) |
| M | cycle the ground surface (changes footstep / landing / slide sounds) |
| click, then mouse | look around (Esc releases the mouse) |
| wheel | zoom |
| 1-4 | take control of Brutus / Flint / Hawk / Tex: their name pops up in the middle of the screen in the game's own HUD font (atlas e0afcd52), with three red chevrons (the game's chevron texture 158f87c1) lighting up on the radar toward their portrait and the HUD open sound; at the cut the close sound plays and the name zooms out to 3x and fades (0.25 s, as in the capture), and the character you left drops back into the squad AI with a follow line ("I'm right behind you", "Following at a distance"...: chatter set f415ab02, from the shared voice bank `ml-sounds/en/common-en.tgz`; their speech icon shows by their portrait while they talk). Not while dead: the death camera hands over by itself |
| R | reload early: **the demo's own key** (the game has no reload button; it reloads by itself on an empty clip, see **Ammo, reload and the crosshair**). The stance's reload clip (`Sc_w1_reload` / `Sc_w2_reload`; upper body on the move, whole body standing); the clip reads 0 (panel red) until the clip's magazine-in event |
| G | use the item in the item box. A thrown grenade (Frag, Energy, Gas, Light, Sonic): hold to charge (the orange meter right of the crosshair, full in 0.6 s, held 0.5 s after the charge ends then fading out by 1 s; the held gun's crosshair stays as it is), let go to throw - the charge sets how far; the count drops at once. A placed one (Roller, Sentry): set down at the feet at the press. See **Grenades** below. The main game starts with 3 Frags |
| E (hold) | use: a gate's wall panel, from in front of it within 2.5 m, looking at it: "Hold [X] to activate panel." shows (the game's glyph line, X's glyph: E is the demo's X; [hints.md](hints.md)) and a blue ring marks its button; held 0.5 s, the gate opens and stays open. See "Gates and their wall panels" |
| T | the next grenade type carried (the demo's key: the recordings don't show the game's button for it) |
| Tab | the item box (the game's B button): tap for the next item carried (each grenade type, Medkit); hold for the item list, the wheel picks one. With a Medkit selected, G heals 80 ("No need to heal" at full health). See "Health pickups" |
| I | in a mission, the MISSION OBJECTIVES menu (the game's BACK; the pad's Back too): the mission's objectives, " (completed)" or " (failed)" after each as its flags say; the game's clock stops under it. I again, Enter or Space (the pad's A or B) close it; while it's up no key or mouse press reaches the game (the closing one included). See [campaign.md](campaign.md) |
| V | the special skill on / off (the game's White button): Tex's Berserker, Brutus's Vengar, Hawk's Stealth, Flint's Targeting. Stamina (the blue bar) pays for it; on needs 10% of it, at 0 it switches itself off. Tex's Berserker draws his other gun into his left hand and fires both; in it Q and R end it instead of switching or reloading. Brutus's Vengar heals him, cuts the damage he takes, charges enemies down when he sprints into them, and turns the view grey with every living character a heat figure. Flint's Targeting widens the crosshair's lock. Hawk's Stealth fades her out over a second: the enemies lose her, the crosshair can't lock her, a shot or an item costs stamina. See [skills.md](skills.md) |
| B | the game's Black button (its Medkit shortcut): only the tutorial's button gates read it for now ("Press [Black] to use Medkit." clears on it); it uses no Medkit yet. See [hints.md](hints.md) |
| Esc | also the game's START for the tutorial's button gates ("Press [START] for game options."): the demo has no pause menu |
| Enter / Space, Q | on a tutorial panel (BASIC CONTROLS, HEADS-UP DISPLAY ...): Enter or Space (the pad's A) accepts it, Q (the pad's Y) skips the tutorial's later panels; nothing else closes it, and no press reaches the game under it (the game's clock stops). See [hints.md](hints.md) |

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
* **Every character type of the data** (128) for the NPC spawner (#180): its levels define
  55; the other 73 come from the 31 level archives that define them, without those levels
  (`Game::load_type_assets`, about 0.9 s more; `BF_TEST_TYPES` picks a smaller set). See
  [testtools.md](testtools.md) and [character-types.md](character-types.md).
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
  spawner (O, in the air from the free camera), any level's buildings and static scenery to put
  down, drawn and solid as in their levels (P, #185), a sky picker (Y), a music picker (U) and a level
  switch (L), which takes the test session to any level and back. On a level the rack and the
  pickup grid aren't there; the rest of this list is. `--test` with `BF_MAP=<level>` starts on
  that level. See [testtools.md](testtools.md).
* **The map editor** (#191): F5 switches to its edit mode, out of the gunplay (no character, gun
  or HUD; a free camera, the right mouse button held looks; the tools at the mouse pointer), and
  F5 again plays from where the pointer is, the camera flying down to the squad and back up after.
  `cargo run --bin bf_play -- --edit` starts the session editing. See [editor.md](editor.md).
* **Backspace** leaves for the front end only when pressed twice within 2 s in a test session (a
  stray key doesn't throw away what was built); the first press says so. `BF_TEST_BACK` still goes
  at once.

## Health, HUD, locomotion, weapons and sounds

Squad: the four squad members are all in play, each with their own health (the characters'
combat-target hitpoints: Tex 115, Brutus 105, Flint 90, Hawk 65) shown on the radar's diagonal
edges. Each bar fills that diagonal's dark channel in the radar frame texture (the frame's own
texels darker than 27/255), starting by the member's portrait, and drains toward it as in the
game. For whoever you control, health also shows in the top-left bar. Grenade blasts hurt everyone in range (the explosion's Damage max, falling
off to nothing at its radius; the thrower takes a flat share, see **Grenades**) and the hurt give a pain grunt (chatter e856009f). Friendly
fire: your shots hit squadmates in the way (the weapon's damage, x0.7 on your own side, by the hit part struck and the difficulty: see **Damage**; they say "I'm hit" or "Careful!"
/ "Stop shooting at me!"), and the crosshair turns green while its centre is on one (see **The crosshair's colours**). At 0 health a
character dies: death cry (chatter ef32191d), the body goes limp as a ragdoll (verlet point
masses on the skeleton, pushed by the hit or blast; its limb bones are 0.09 m spheres that don't pass through each other, and every bone lies on the level's own floors, kept as far above them as the body is thick round it (`flesh`: of the skin each bone moves most, the distance from the bone's line that 80% of it is within, 0.04-0.2 m; a flat 5 cm let a thigh or the chest sink into the floor), and is kept out of its walls - `BF_RAGDOLL_LOG=1` prints the closest limb pair and the lowest bone; the pose takes only the bones' turns from the simulation and keeps their rest offsets, so limbs don't stretch or twist; the ragdoll starts where the body is drawn, sole lift included (see "Standing on the floor"); the torso moves as two solid pieces (hips; chest with shoulders), stiffly joined at the waist, with the head held to the chest; knees and elbows (found by their Bip01 names) bend but don't fold past about 120 degrees or straighten past straight, and only one way: each joint keeps to its side of the line from the upper bone to the end, in the torso's own frame (hips for knees, chest for elbows), the side it was bent at death; a nearly straight limb uses knees forward, elbows back (forward from the toes); bodies keep 98.5% of their speed per substep, lose 60% of their sliding on the ground, and rest once still - but not on their side: a body come to rest propped on a shoulder and a hip (its chest's left-right axis more than 0.6 upright) is rolled on, its upper shoulder pushed 2 m/s toward its back (or front, if it leans that way), up to 3 times (the demo's choice). Shots hitting a body on the ground (dead or knocked down) push it: the first bone the shot passes within its thickness of is brought up to 3 m/s along the shot, and bones within 0.6 m less, so it jolts and rolls (a burst doesn't add up past that speed, so it doesn't drive the body across the ground); it wakes the body to settle again and it bleeds where it's hit, as the living do (blood effects, hit sound, a splat on the ground) (`BF_RAGDOLL_LOG` prints each body hit and the pelvis's position); an enemy of a type of its own (#129) has the game's ragdoll instead: bodies for its hit parts only, the rest rigid as posed at death, and the game's knee, elbow and hip limits ([enemies.md](enemies.md), "Death"); `BF_TEST_DIE=<s>` drops the player dead at that time, once; `BF_TEST_DIE=1` at 1 s). Both their guns drop, the one in hand and the one on their back (the ragdoll can't feel a stowed gun, so it propped bodies on their side): each falls from where it was, thrown by half of what killed them plus a 1.5 m/s hop, and tumbles as a loose object (see "Pickups are loose"). Then the portrait greys out and the blip leaves
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
22 46 86, ours 17 51 91). Its baked bars are cleared, so the bars' empty part is see-through as in
the game (#35 take10: the sky shows through the empty stamina channel; it was the picture's navy
before), and the live bars drawn over them (rows 22–27 and 35–40). The bars are the game's bar
widget's length (FUN_000b7050: value / 150 × 249 units), 1.66 units per hit point of the
character's maximum health (`combat-target hitpoints`: Tex 115, Brutus 105, Flint 90, Hawk 65;
`BF_DUMP_HITPOINTS=1` lists them), from x 73.0, at least 2 units above 0: Tex's are 190.9 long,
x 73.4–263.9 in the demo and in #35 take01 (it was 1.6 per hit point from x 74, 3.5% short). The
stamina bar's value is stamina / max-stamina × max health, so it's as long as the health bar at
full. The frame's middle stretches with them and its right cap follows their end, and a squad
switch resizes it (the picture's right cap ends 12 units past the bars, the game's frame ~9.4:
the demo's frame is ~3 units longer on the right, #35 takes 01, 06, 19). The stamina bar's
colours, trail and low-stamina blink, and the skill's name under it: [skills.md](skills.md).
Without the tutorial's picture, the frame below is used.

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
recharging gun; every weapon listed with names for 2.5 s after a switch, the held one in orange; in Tex's berserk two rows, the gun in hand over the second hand's, both orange, [skills.md](skills.md)), radar with the squad's four portraits round it (Tex top, Hawk left, Flint right, Brutus bottom),
grenades bottom right, and the held weapon's own crosshair (its definition's `reticule-prefix` texture, e.g. f4ea7a92 for the MK-ASLT), 64 units square, above centre (y 192 of 480) - aiming and shots go
through it; red, green, yellow or blue as the game tints it (**The crosshair's colours**, below). Layout is in the game's 640 x 480 screen (measured from an xemu capture) inside a
centred 4:3 area; textures are the game's own from `common.tgz` (stored upside down, drawn
flipped). The HUD is drawn by the main camera's child, the Vengar view's heat camera
(play_vengar.rs, marked `IsDefaultUiCamera`), over the world and the heat figures, whatever the
cameras' order. `BF_NO_HUD=1` hides it. Ammo, the switch hint and the crosshair's
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
    Where a level starts the squad from its own placements (a spawn-trigger's template, or the
    squad placed in the world: [triggers.md](triggers.md), #220), the ammo they carry
    (`<inventory><h_10dd6db2 name count>`) sets those types instead: the tutorial's Tex 500 High
    ROF, take03's "48 / 500".
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
  ~0.5 s after Y, back at ~0.95 s). It stays, unchanged, while a grenade throw charges and
  through the throw: a thrown grenade has no crosshair of its own (#119; `todo/119` take01).
  Aiming and shots go through it (play.rs `CROSSHAIR_UP`). The
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

* **A gun in the second hand** (Tex's berserk, [skills.md](skills.md)): the character's reload,
  FUN_00120d40, returns at once while the inventory holds one (+0x198), before it touches the
  reserve. So in berserk the gun in hand doesn't reload (#35 take05: 0 / 200 for 8.7 s), an empty
  one clicks dry under the trigger, each gun with its own empty sound, no "switch to" hint shows
  (FUN_00120d40 posts it), and a reload asked for (R) waits for berserk's end. Both guns' rows
  show in the weapon box, both orange.
* **Flint's targeting on** ([skills.md](skills.md#flints-advanced-targeting-124)): the reload also
  asks whether the character is free (FUN_00117210, 0x120e16), which ends with the skill's
  vtable +0x14, and hers says no while it's on (0x13b980). Her empty clip and R end it first, so
  the reload starts in the frame after the off (`BF_TEST_CLIP=1`: off at 5.13 s, the reload at
  5.20). The same rule holds back a weapon switch, a grenade thrown or placed, and the medkit.

Test hooks: `BF_TEST_RESERVE=<n>` starts every ammo-type's reserve at n; `BF_TEST_CLIP=<n>[,<n>...]`
starts the player's clips at n (one value for every weapon, or one per weapon slot, the last for
the rest: Tex's `99,4`); `BF_TEST_TRIGGER=<from>-<to>[,...]` holds the trigger over those
windows (s; overrides `BF_TEST_FIRE`); `BF_TEST_SWITCH=<s>[,...]` presses the weapon switch once
at each time (it owns the switch for the run: Q does nothing then); `BF_TEST_RELOAD=<s>[,...]`
presses R once at each time (it owns R for the run); `BF_TEST_KNOCK=<s>` knocks the player down
then, once; `BF_TEST_CROUCH_KEY=<s>[,...]` presses Z once at each time;
`BF_AMMO_LOG=1` prints the reserve, each reload, dry shot (and the second hand's), recharged
round and the hint.

### The crosshair's colours (#141)

`src/bin/play_reticle.rs`, drawn by `play_hud.rs`. Read from `default.xbe` (each function is
cited in the code and has a row in [code-map.md](code-map.md)), checked against the takes in
`todo/141-crosshair-colours/` (takes 01-09 and xemu's live reads in their notes).

The crosshair is drawn in one of four tints, worked out every frame and switched on the frame
its condition changes (no fade, as in every take):

| Tint | When | The game's colour (FUN_000bbd70) | Drawn as |
|---|---|---|---|
| red | the reticle holds a **lock** on a hostile | 160 32 32 | 152 32 32 |
| yellow | no lock; the first thing under the centre has **radar-contact 2 or 4** | 200 200 0 | 190 190 0 |
| green | no lock; the first thing under the centre is a living **teammate** (not a 2 or 4) | 68 220 68 | 66 209 67 |
| blue | anything else | 50 150 255 | 49 143 243 |

At alpha 229 / 255 (FUN_000c6280's 0.9 x 255; it was 0.95). The console blends in the stored
sRGB values, Bevy's UI in linear light, so each is drawn as the colour that gives the game's
result over a dark ground, as the grenade meter's orange is. The demo's own green on a squadmate
(77 255 89, no source) is gone: the game's is the code's 68 220 68, by its own rule.

* **The lock** (the target selection's FUN_000bf7a0, the best kept by FUN_000bfe40): any living
  character but the player when
  * it's hostile by the team table (0x3ffd80, the row of the player's team): the squad locks
    teams 1, 7 and 8; a neutral (2) or a teammate never;
  * its **aim point** is within reach of the gun's muzzle: game-options' `<reticule
    h_0c3eb4ff="0.7">` x the gun's reach (FUN_00232520): the bullet's `range`, x0.9 for a burst
    of pellets (the Bower 20), and for a mesh projectile its second timer x its speed added (and
    its range again with MIRV on explode): LZR-50 63 m, Jax-iP 28 m, Therm Sweeper
    0.7 x (20 + 20 + 5 x 30) = 133 m. With a gun in the second hand (Tex's berserk) the smaller
    of the two guns' reaches counts, measured from that gun's muzzle (0xbf82a-0xbf8aa, then
    FUN_000bff40 at 0xbfb14): with the LZR-50 in hand and the Jax-iC second, the Jax-iC's 42 m
    from his left hand (`LockGun`); the box below and the centre ray stay the gun in hand's
    (0xc57e1). The aim point is the point of the character's **aim
    capsule** (its vtable +0x134, 0x121d00) nearest the crosshair's ray (FUN_0015b450): from 40%
    of the way from its "Bip01 Head" node to its root node (the hips) to half that length below
    the hips, 0.15 of it thick. So a sweep locks as the body's middle, chest to thighs, comes
    near the centre, not one point of it;
  * projected on the HUD, it's within a box round the crosshair's centre of 33.6 x s units each
    way (`<reticule h_f079aebd="0.14">` x 480 / 2; across, x 0.75 x 640 / 2: square on screen),
    s the weapon's `h_e1e41c2d` (`WeaponDef::lock_box`: LZR-50 0.85, Jax-iP 0.6, Therm Sweeper
    0.8, the iKhan-GPL 0: never), **x1.2 for the one locked last frame** (0xbfae9): the LZR-50
    locks within ±28.6 units and keeps it to ±34.3;
  * a ray from the camera to it (256 m) meets it before the world or another body: a squadmate
    walking in front hides it.

  Of those, the most central wins: the lowest depth x (0.3 + 0.8 (1 - score)), the score a
  smoothstep of how near the box's centre it is.
* **The centre ray** (0xc5720), with no lock: the first body or placed object the ray through
  the crosshair's centre meets within the gun's range (256 m with no gun) decides. A hostile is
  locked at once in the scope (and with no gun, or a gun whose lock reach outruns its range: the
  Therm Sweeper's 133 m against its 20 m); out of the scope it stays blue (take02 and take03: out
  of reach, blue), whatever its radar-contact: for a hostile the radar test is skipped (0xc5943 ->
  0xc5a3e). A living teammate (the same team, one not hostile to itself) is green; then anything
  else with a radar-contact of 2 or 4 (a teammate, a neutral, a placed object) is yellow. So red
  beats yellow and yellow beats green. No box: green and yellow only while the centre is on the
  thing.
* **Radar-contact** (an object's +0x7a, `radar-contact-type` h_1905f381; RCONTACT_ 2 OBJECTIVE,
  4 OBJECTIVE_SECONDARY): every character has 1 (`Player::radar_contact`). A level's objects take
  the placement's own `h_1905f381`, else their type's, then the radar-triggers that operate at the
  level's start (signal 13, FIRST_UPDATE) set theirs (`bf/level.rs`: `Placement::radar_contact`,
  `Level::radar_triggers`, for the radar's #160 too). On e34 that's the five missile racks and
  three h_e8589e7e devices at 4 and the device h_00bc61ab at 2, exactly as the game had them
  live; no squad deathmatch map has any. The centre ray tells which placed object it met from
  the arena (`Arena::ray_object`).
* **The guns aim at the lock** (#124): with a lock the reticle gives the controlled character its
  point (0xc5b90 -> FUN_0011e9d0, char+0x804): the centre ray's point when it meets the locked
  body (0xc59fe), else the lock's aim point (0xc5330 -> 0xc5170). The character update aims each
  of its guns there (0x11aa20 -> 0x22dc60) and each muzzle's shot goes to it (FUN_0022e5b0):
  `Reticle::aim`, play.rs `Player::lock_aim`, the arms turned onto it and the shots sent to it
  (from the eye to it in the scope), the second hand's gun's too, a missile's aim point too. So a
  burst fired with the crosshair red hits the locked body (the LZR-50 4 of 4 at 9 m with the
  crosshair inside its box; blue, 49 units off, 4 misses). Without a lock nothing changes: the
  shots go to what the crosshair is on. The AI aims at its own target.
* **Flint's Advanced Targeting** ([skills.md](skills.md#flints-advanced-targeting-124)): while
  it's on her skill sets the box: 0.4 of the screen each way (her +0xc4, 0x13cd30, writes the
  selection's shares) at scale 1.0 (her +0x100, through FUN_000c5d20 each frame): 128 x 96
  units round the crosshair; her +0xfc fades the crosshair's alpha (0.9 x (1 - her marker's
  fill), `Reticle::alpha`); her rifles' range is x1.25 (FUN_002324c0 through her +0x54), the
  centre ray's and the lock's reach with it.
* **Hawk's stealth** (#123, skills.md): a character less visible than 0.1 (its skill's +0xe4,
  `skills::visibility`; FUN_000bf6d0, `LOCK_VISIBILITY`) is neither a lock candidate nor locked
  under the centre (FUN_000bf550): a stealthed Hawk on another side leaves the crosshair blue.
* **Not in:** the line-of-fire ray (0xc54d0: an obstacle between the gun and a locked target
  would show its own colour after two frames; never filmed); the alpha's blink; the
  scoped lock box (0x11cfb5: once the scope-in's 0.85 s timer, char+0x7c8, has run out, the box
  scale is the skill's +0x58 x the gun's, 0.2 for the squad, 0.8 for Flint, where the demo keeps
  the gun's); the crosshair moving onto a lock (0xc5a50, FUN_000c5e20 at the skill's +0xf8 speed:
  the takes' crosshair stays put); a moving target's lead (FUN_0022e5b0, for a bullet that isn't
  instant: the demo's shots reach their target at once). The demo's bodies are cylinders, so the centre ray
  doesn't slip between an arm and the body (take04's 2-3 blue frames on Flint), and it passes
  the dead (the game's meets a body and tests its radar-contact). A choice of the demo's: the
  candidates are tried in the squad's order: the game walks the world's target list (0xba1c0),
  whose order isn't traced, and with its running best scores that order decides between close
  candidates. (The second hand's reach, a choice here before #121, is the game's now: the lock
  above.)

Test hooks: `BF_RETICLE_LOG=1` prints the tint, the lock and what the centre ray is on at each
change (`=2`: every frame), with each hostile's aim point on the HUD, the box it's tested
against and why it isn't locked, and the camera; `BF_TEST_SWEEP=<deg>|<from>,<to>,<deg/s>[;...]`
turns the camera (degrees, + to the left; with `BF_TEST_GOTO` from its start-to-goal heading);
`BF_TEST_RADAR=<object>,<value>[;...]` sets a level object's radar-contact by its name as the map
starts (sdm_e34's rack: `h_128f3de2,4`). On the test map `BF_TEST_NPC` takes `neutral` or a team
number, `BF_TEST_NPC_TEAM=<s>,<n>,<team>` sets one and `BF_TEST_NPC_RADAR=<s>,<n>,<value>` gives an
NPC a radar-contact ([testtools.md](testtools.md)).

Checked on the test map, Tex kneeling with an NPC 9 m ahead (`BF_NO_SQUAD=1
BF_TEST_GOTO=4.5,20,4.5,19.5 BF_TEST_CROUCH=0.2 BF_CAMERA_PITCH=-0.12 BF_TEST_WEAPON="Confed
LZR-50"`, `BF_CAPTURE`):

* take08's sequence on one NPC (`BF_TEST_NPC=0.5,tex,enemy,dummy,4.5,11 BF_TEST_SWEEP=1.1
  BF_TEST_NPC_TEAM="3,1,neutral;9,1,friend;12,1,enemy" BF_TEST_NPC_RADAR="6,1,2;9,1,1"`): red,
  blue, yellow, green, red, each a one-frame step. The arcs against take08 (the 15 strongest
  pixels' median; the mean of the arc's pixels): red 120 25 24 / 100 31 29 (the take 125 26 32 /
  95 22 26), blue 43 114 196 / 38 89 148 (44 114 201 / 35 85 142), yellow 150 151 1 / 116 116 17
  (156 161 16 / 118 126 20), green 52 168 52 / 44 129 45 (64 184 56 / 48 134 46). The peaks run
  a few levels low: the game draws the 64 x 64 texture 1:1 at 640 x 480 and xemu scales the
  picture, the demo samples it at 1.5x (so every HUD texture); the take's yellow leans green
  from the video's colour encoding.
* A sweep like take01 (`BF_TEST_SWEEP="20;2,9,-6;11,18,6"`): red from +27.5 units (the
  LZR-50's ±28.6) to -34.8 (its ±34.3 kept), and back from -28.3 to +35.9: it locks nearer the
  centre than it lets go, as take01 does.
* A friend (`BF_TEST_NPC=0.5,flint,friend,dummy,4.5,14`, `BF_CAMERA_PITCH=-0.17`): green only
  while the centre ray is on her, a narrower window than the red's. With the squad in play,
  squadmates walking across the crosshair turn it green, and one walking between Tex and the
  enemy hid the enemy (blue) until it passed.
* Precedence: an enemy with radar-contact 2 stays red; a friend with 2 is yellow; with 1, green;
  shot dead (`BF_TEST_TRIGGER=12-12.6`), blue.
* Reach: the Jax-iP (28 m) locks an enemy at 25 m and not one at 33 m with the centre on it
  (blue; the log: "out of reach: 32.7 m from the muzzle, reach 28.0"). Flint's MK-ASLT on an
  enemy 60 m away: blue unscoped, red in the scope (`BF_TEST_AIM=1`: the centre ray's lock). The
  Therm Sweeper locks it at 63 m (box 26.9) while its centre ray stops at its 20 m range.
* The second hand's reach (#121, standing: `-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.535,19.5
  BF_CAMERA_PITCH=-0.07 BF_TEST_NPC=0.5,tex,enemy,dummy,8,-30 BF_TEST_SKILL=2 BF_RETICLE_LOG=1
  BF_TEST_WEAPON="Confed LZR-50;Jax-iC"`): the enemy 50 m out is red with the LZR-50 alone
  (0.60 s, "locks", 53.6 m deep) and blue from the Jax-iC's draw into Tex's left hand (2.60 s:
  "out of reach: 50.9 m from the muzzle, reach 42.0"), as the frames show at 1.3-2.0 and 3.0-4.0 s.
* Level data: on e34 the level's radar-contacts load (`BF_RETICLE_LOG` lists them as the map
  starts) and the crosshair is yellow on the missile rack h_00815cce from take06's spot (`BF_MAP=e34
  BF_NO_SQUAD=1 BF_TEST_GOTO=-44.68,17.61,-44.21,17.77 BF_TEST_CROUCH=0.2 BF_CAMERA_PITCH=-0.2
  BF_CAMERA_DISTANCE=7 BF_TEST_WEAPON="Confed LZR-50" BF_TEST_SWEEP="15;1.5,11.5,-3"`), blue on
  the pillar beside it.

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
  the scope); a dry click (`play_ammo.rs`) and the Charge's shot cooldown use the same
  `shots::period` (FUN_002327f0, FUN_0022dc00).
* **Taps and the raise.** A press is a request that outlives the button: the holder's +0x260 bit
  3 (8) sets the fire flag of the gun in hand and of the second hand's each frame
  (0x122614-0x122655) until an animation event clears it (0x0f8408d5, FUN_00117660 at 0x117b12;
  which clip isn't placed), so the demo holds a press until its first shot (`FireState::pull`):
  an 80 ms tap fires. With two guns (Tex's berserk) it holds it for each until that gun's first
  shot, so a tap fires both, the one still cooling down when it's ready (#121: a tap at 3.98 s
  fired the LZR-50 at 4.07 and the Jax-iC at 4.27, after the button came up), and while either
  is owed its shot an empty one runs its dry cycle (the request sets both flags), but the demo
  clicks and syncs the LZR recharge only while the button is down (`u.fire`, play_ammo.rs); the
  game's FUN_002327f0 clicks on every such cycle, so an empty gun after a short tap stays silent
  in the demo. From idle the gun is raised first (the demo's 0.2 s, which the takes' 0.19-0.24 s
  press-to-shot fits); after a shot it stays up 4 s
  (`READY_HOLD`: FUN_00123f60 keeps +0x7dc at 4 or more, medium confidence), so the takes' taps
  2.2 s apart fire at once (0.07-0.11 s). Checked: 80 ms Bower taps at 2, 4 and 6 s fire at
  +0.20, +0.13 and +0.13 s (15 fps capture steps); at `BF_CAPTURE_FPS=60`, +0.20 and +0.03 s.
  The shot comes 0.20-0.23 s after the frame that sees the press; at 15 fps a
  `BF_TEST_TRIGGER` window starting on a frame's time (e.g. 3.0 s) can be seen only on the next
  step (3.067 s), so that tap's shot logs at +0.30 from the window's start.
* **The burst counter** is cleared by a reload starting (FUN_0022daf0), a dry click
  (FUN_002327f0) and a switch (FUN_0022dc00), as in the game.
* **The seed** of a shooter's fire loop is the demo's own hash, character x 7919 + NPC number x
  104729 (`play.rs` `shot_seed`), in wrapping arithmetic (#229): an NPC numbered above ~41,010
  overflowed a u32 and panicked a debug build. The seeds below that are unchanged, so the shot
  patterns are too. Checked by `cargo test --bin bf_play shot_seed`.
* **Bursts and pellets**. A trigger cycle is h_e4076713 shots, h_e6c60892 s apart. At 0 s apart
  (the Bower 20's 6) they all go in the same frame for one round and one report, and each after
  the first is turned from the one before by up to h_e704fd69 degrees of yaw and of pitch
  (FUN_0022e1d0: the pellets walk away from the first; medium confidence on the walk).
  The round is taken on the cycle's last shot: FUN_0022fe00 excuses every shot while the
  counter is running, and only once FUN_0022f0e0 has cleared it (the 6th) does the clip drop.
  So the clip stays put through a burst and every shell is 6 pellets, the last of a clip too;
  the reload then starts 1 s later (the takes: Bower +1.0 s).
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
  unset: the squad's `accuracy-bonus` h_072837bb flag h_ec5337fb is false in the levels; some
  enemies have 75 or 90). A character a spawn-trigger made from a placement that sets
  h_ec5337fb takes its h_072837bb instead (#227, `Trigger::bonus`: the tutorial's Red Hand +25,
  S = (100 - clamp(A + 25, 0, 100)) x 0.03). So the MK's first shot spreads +-0.33 degrees and its 14th on in held
  fire +-2.3; the Minigun's +-1.2 to +-1.8 over a clip; the Bower's aim +-0.5 before its
  pellets' walk.
  * Against the takes: the clusters measured in screen units, read with a 45 degree vertical
    field of view, came out 1.5-2x tighter than this. The data's two player cameras
    (objecttypes-common `<camera> fov="90"`; a debug dump labels a camera's field "fovh", so
    horizontal) give ~74 degrees vertical at 4:3, and under that the MK, Minigun and Foley match
    the takes within about 10%: the likely explanation, pending xemu's read of the game's FOV.
  * The Bower is still ~2x too wide horizontally at 74 degrees (its vertical matches). Open
    question: the pellet walk (FUN_0022e1d0, read as each pellet turned from the one before);
    xemu's Bower pellet shot is queued.
* **Damage falloff** (FUN_00230040 -> FUN_00223780). An instant hit's damage is scaled by the
  Damage's h_fb124e6c mode at x = 1 - distance / range: 1 x, 2 x^2, 3 full to half range then
  2x, else none. Only the Bower has one (2): 0.60 at 9 m of its 40 m range. It's applied to the
  squad's, the NPCs' and breakable scenery's hits. The rest of the take's ~1.3x on the trooper
  is the difficulty step and the struck hit part (#127): see **Damage** below.
* **Range in the holder's hands** (FUN_002324c0): a shot's ray and its falloff run over the
  bullet's range through the holder's skill (+0x54): unchanged, but x1.25 for Flint's rifles
  (WTYPE_RIFLE, on or off; #124, `skills::weapon_range`): the L-Shot-75's 180 m is 225 m in
  her hands.
* **Holes** (`play_fx.rs`, `HoleRequests`). Every gun shot that meets the world leaves its
  bullet's h_06a27365 decal (0.3 x 0.3 m holes and scorches, read as full sizes; 25 s, fading
  over the last 5) laid on the surface it met: the map's collision triangle
  (`Arena::ray_normal`) or, on the flat test floor, the ground or a pillar's side. Up to 160 stay
  (a guess).
* **Muzzle effects**. The weapon's h_fd88830d effect type plays at its muzzle hardpoint once a
  frame in which it fired (the MK's orange flash and grey smoke); it replaces the plain flash
  quad on guns that have one. It takes the hardpoint's own frame (#73): FUN_00232060 attaches
  the effect object at the hardpoint. Every weapon's muzzle hardpoint has the barrel along its
  -z (unturned, or half a turn about y on the models built facing +z) and the flash emitters
  are turned -90 degrees about x to fire along it (the MK's sminigun_muzzle.emt), so the
  flashes now leave the barrel forward; turned +y along the barrel (before) they fired across
  it. One exception: the Psionic Blast's (ORIENTATION w x y z 0 0 0.707 0.707) puts its -z
  down the model's -y; its effect (seer_fball_mzl_purple) isn't checked against a take. The
  frame was found on the Therm Sweeper, whose back-blast dust (mtrail-dust, 4.3 m along +z,
  2 m down) hung 4.3 m above Tex and now lies behind the launcher, where the follow camera
  doesn't see it (the takes show none from behind Tex); by the game's emission count it's one
  brown card (see **Missiles**, the effects' count). An effect parented to the gun now starts on
  the shot's frame, placed from the gun's last placement (`ale_fx.rs`'s emit; it waited a frame
  before).
  * **Facing 3** (#73; `src/ale_fx.rs`, `FACING`): an appearance flagged AngleFadeOut (the
    demo's "perp") whose GeneralApp_FacingType is 3 is drawn as the game's particle vertex
    shader (0x3b8bf8) builds it: a quad stretched along its particle's motion and turned about
    it to face the camera. In the hardpoint frame, lying flat across its y, the sphere
    emitters' glows (the A10's, the Saryl-45's, the Jax-iC's) were edge-on lines; they're round
    again, as before it. Every other appearance keeps the demo's rules.
  * **The light**: a muzzle effect type with a light-effect plays it as a point light at the
    muzzle, as the blasts' lights are played (`src/ale_fx.rs`), in place of the demo's own point
    light (`HeldWeapon::data_light`); a gun without one keeps the demo's. Each kind of gun names
    its own: light_muz_minigun (h_08b5bbab, 0.2 s growing 0.28 -> 2.66 m) on the ballistic guns
    (MK, Minigun, Bower, Foley, Kman, L-Shots), light_muz_laser (h_1a96e11e, 0.2 s, 0.26 ->
    2.53 m) on the LZRs, light_muzzle (h_e7e8d51b, 0.24 s, 0.5 -> 4.6 m) on the Therm Sweeper and
    the Sweeper V, and _bio, _cutter, _sonic, _particle, _rail, _seer and _energy on the others;
    none on the iKhan-GPL. Its seed is made from the effects', so the gun's random stream
    (spread, frame jitter) runs as before it. It's lit from the shot's frame (`BF_SHOT_LOG`'s
    round, the effect and the light's first update share it) but draws from the next: a point
    light made that frame shows a frame late in the demo, where the demo's own light (an
    existing one turned up) showed at once. Not fixed.
  * **In the controlled character's own scope** it's drawn 3.8 m from the eye with its axis
    along the line of sight, where the take shows it (`SCOPE_FLASH_REACH`, `SCOPE_FLASH_AT`; the
    demo's rule): at the muzzle, a few tenths of a metre from the scoped eye and magnified, its
    sparks covered half the view. The scope-in code (0x1223da, FUN_001249b0) doesn't move or
    hide it, and nothing else that does was found. There it keeps, on purpose, the turn it had
    before the hardpoint frame (+y along the sight) and the demo's perp rules: the fit below was
    made with them, and the scoped MK draws as on main (frames 14-39 of
    `BF_TEST_WEAPON=MK-ASLT BF_TEST_FIRE=1 BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12` differ
    from main's by at most 7 of 255). No light there, the demo's old rule and a choice: from the
    gun under the eye the data light washed the whole scope view orange for a frame (the scoped
    MK pitched down, +29 red on average), and no take shows the game's light in the scope.
    * The take (63/take01 at 28.77-28.85 s, the bright pixels that changed, 640 x 480 units): a
      soft orange ball, centroid 95-104 right of the crosshair and level, 124 x 144, its left
      edge (x 344) clear of the crosshair (303-336), gone in ~0.1 s.
    * Ours (frames 21 and 24 of `BF_TEST_WEAPON=MK-ASLT BF_TEST_FIRE=1`): centroid (98-102,
      -3..+4), 145-155 x 78-97, left edge 22-29 right of the crosshair's middle, clear of it.
    * In the scope the effect leaves out its flat flash quads (perp quads lying along its axis,
      `AleEffect::hide_flat`, the demo's rule): seen from the eye they lay as bright ellipses
      under the star. The star itself is the ALE rendering's (#104), not the game's soft ball.
* **Tracers** (FUN_002317e0). An instant ray carries its flight effect only when its counter is
  0, which then restarts at h_eeb9e75a: the MK every 3rd shot, the Minigun every 4th, the rest
  every shot. The tracer is now turned along the shot (ALE emitters fire along their +y): it
  flew straight up before, the thin vertical line over the Minigun.
* **Casings** (FUN_0022eee0 -> FUN_00231b10). Each trigger cycle owes h_fdc93b33 casings; each
  whole one throws the h_19b21bf5 object (the casing h_16415157's model) from the h_ea1abbbe
  hardpoint, 5 cm back along its z, at 2-3 m/s along its z turned a quarter turn about the
  vertical, plus up to 0.5 m/s on each other axis. MK, Minigun, L-Shot and Foley; the Bower has
  the hardpoint but no casing object. Their 2 s life, gravity and bounce are guesses.

* **Two guns at once** (Tex's berserk, [skills.md](skills.md)): the fire loop steps each gun as
  its own weapon object in the same game frames (`FireState::run` over `Gun`s: its cooldown,
  burst counter, empty cycles, muzzle effects and casings per hand; accuracy and tracers per
  weapon as before), both on the one trigger. Each shot carries its own gun's Damage,
  damage-type and ammo-type to the body it hits (it used to take the gun in hand's for every
  shot of the frame). One gun's fire is unchanged: against main, the autopilot's shot and ammo
  logs as Tex and as Brutus, and the LZR-50's held fire, taps and dry cycles on the test map
  (`BF_TEST_TRIGGER=1-4,5-5.08,5.5-5.58,6.5-6.58`, with and without `BF_TEST_CLIP=6`), are the
  same line for line once each pellet's new `hand: 0, ` is taken out of the shot log, and the
  test map's frames pixel for pixel.

Test hooks: `BF_TEST_WEAPON=<label or hex>` (e.g. `"Bower 20"`, `"RVG50 Minigun"`, `0c3db625`)
puts that weapon in the controlled character's slot `BF_START_WEAPON` (default the first) if the
map's data has it (the test map has every hand weapon); a list, `<a>;<b>`, fills the slots in
order from the first (#35's Tex: `"Confed LZR-50;Jax-iC"`); `BF_SHOT_LOG=1` now also prints each
pellet (its hand, frame, spread, walk, tracer) and the gun's accuracy; `BF_TEST_FIRE=hip` fires
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
  walking up to ~15 degrees from the first; holes on the pillar and on sdm_e34's walls. (Its
  muzzle effect along the barrel in a side view, `BF_CAMERA_DISTANCE=2.5 BF_VIEW_YAW=1.3`, was
  #115's check, before #73 put the muzzle effects in the hardpoint's frame: see **Muzzle
  effects**.)
* Casings land to Tex's right on the floor; the muzzle smoke and flash at the Minigun's barrel.
* With `play_ammo.rs` (#114): the Minigun reloads after its 80 rounds and fires on; the Bower
  with `BF_TEST_CLIP=2` fires two 6-pellet shells, reloads 1.0 s later (reserve
  80 -> 68) and fires full shells again; with `BF_TEST_RESERVE=0` it clicks dry about once a
  second instead.
* The main game on sdm_e34 (`BF_AUTOPILOT=1`, 340 frames) is pixel for pixel the same as
  before until the autopilot's first shot.

### Missiles: the Therm Sweeper (#73)

`src/bin/play_missile.rs` (the camera shake in `src/bin/play_shake.rs`), the bullet's data in
`weapon::MeshBullet` (`src/bf/weapon.rs`), from default.xbe's projectile class
(0x226000-0x229000) and the weapon type's damage handler (0x22d0b0); every function cited here
is in [code-map.md](code-map.md). The measurements are the tester's spec,
`todo/73-therm-sweeper/spec.md`, from takes 01-07 there; takes 08-23 (a real reload,
self-damage at a wall, homing on two enemies and an off-line one, the open air, a side view,
cover) came after it and are compared under **Verified**.

* **Which guns.** bullet-type 2 is BTYPE_MESH_PROJECTILE (the enum at 0x3bd63c): FUN_0022f1d0
  launches the bullet's mesh-name object (FUN_0022fb50). Those steered at a steady speed
  (h_0b03ece5) that split (a `<mirv>`) fly here: the Therm Sweeper. The Sweeper V (steady, no
  MIRV) keeps the plain shot until #72 checks this path against its own takes
  (`todo/72-sweeper-v`); the other bullet-type 2 weapons (the iKhan-GPL's lobbed cluster, the
  creatures') fly under their physics body's gravity, which isn't traced, and keep it too.
* **Launch**, on the fire loop's frame: the shot, the clip drop, the fire sound ed761bae and the
  muzzle effect in one game frame, as FUN_0022f2a0 does them (take21's side view: the clip digit
  and the missile at the muzzle on the same frame). The missile object h_e9327576 (model
  h_07d84a52; attached effect h_194dc78e: mtrail, mtrail#1, mtrail-flare and the hiss 0021996c)
  leaves the muzzle along the shot after the accuracy turn (+-1.23 degrees at accuracy 60) at
  the bullet's 30 m/s. Its target is the character on the crosshair's line before the world,
  if it's hostile (weapon +0x360, an inference: a squadmate in front of a hostile hides it, as
  it hides the crosshair's), its aim point the crosshair's world point (weapon +0x1ec). Life =
  range / speed = 20 / 30 = 0.667 s, the straight phase h_ea5327e6 x that = 0.1 s, then the
  second timer h_f0a56297, 5 s (FUN_00227dd0).
* **Flight** (FUN_00226cc0, in 30 Hz ticks: the move first, then the rest). Steering
  (FUN_002266d0): each tick removes half the velocity across the steering direction and pulls
  the speed 15% of the way to 30 m/s. The direction is along its velocity in the straight
  phase, then its target's root or its aim point. **The root** is where it homes on a character
  (`root_point`): FUN_00226cc0 refreshes the aim point each tick from the target's scene node
  (vtable +0x40, 0x10fda0; 0x2270fe), a character's model root node, which FUN_0023d4f0 puts at
  the feet + the root-height of the clip last started: the hips, ~1 m up (Brutus's rp_idle
  0.98, the Shrikes' idles 1.014 and 1.121), not the feet. An aim point within 1 m, or over 60
  degrees off its heading, is given up for one ahead along its velocity (velocity x 200: 6 km,
  0x227038), and the target with it; that test runs before the fuse, so a NEUTRAL child that
  gives up has no fuse and flies on to contact or its timeout. The trail's user parameter
  follows life / start (FUN_00228c10). No gravity: measured, not traced (take20's children
  climbed straight at ~8.5 degrees for 168.6 m in 5.58 s).
* **The trail's puffs** rise in place: the air fields linked to them, mtrail-rise (0.3 m/s up)
  and mtrail-rise#1 (1.0 m/s up, Approach 1.66 taken as 1), now run (`AIR_FIELDS`,
  `src/ale_fx.rs`). take21's side view shows a cloud round the missile from 0.1 s, then a line
  of puffs it leaves behind; without the fields their cones threw them 0.9 and 14.5 m/s back
  along the missile's way, past the holder.
* **The split** (h_11f515c5 = 2, WM_MIRV_ON_EXPLODE; FUN_00227f80), when its life runs out, 20 m
  out. The parent goes without a blast. Its 2 children (h_fe04300d) leave beside it, offset by
  the two missiles' bounding radii (0.32 m) along a side axis rolled at random (180 degrees on
  for the second), turned spread-max x 90 = 9 degrees toward it, at 30 m/s, living 0.85-1.15 x
  0.667 s and then 5 s more. The hostiles the split finds within 20 m and 30 degrees of the
  parent's velocity (its body's, vtable +0x54 = 0x159d70: the split builds its frame from it
  with FUN_0001d0c0) are dealt out to them in turn, the most central first; without any, each
  aims 200 m along the parent's velocity and comes back near parallel. **A child is NEUTRAL**:
  it keeps the projectile constructor's side 2 (FUN_002264e0 at 0x2265ec), since the split calls
  FUN_00227dd0, which copies the owner's team, before it copies the owner (0x2282bf, 0x22834a),
  and row 2 of the hostility table (0x3ffd80) is hostile only to PC_NO_TEAM (7), a side no
  character of the demo is on. So its own search (FUN_00226b80: once at the straight phase's
  end, target or not, 0x226d8e-0x226da7, and every tick while it has no target, 0x226e02; its
  timer 0.25 s; 60 degrees off its heading) finds nobody, which keeps the target it has: its
  only target is the one the split dealt it, and one that gives that up flies on along its
  velocity. The split searches with the parent's side, the shooter's (0x228199). The search
  (FUN_002237c0) measures its cone to each character's root and needs a clear ray to it,
  through a world query (FUN_000df0a0, flags 0xfffffffa) that takes the terrain and some
  dynamic objects but none of the level's placed geometry: the demo's ray meets only the
  terrain (`Arena::ray_terrain`); at the hips the ground they stand on isn't in the way.
* **Proximity fuse** (h_0bafbd6f): every 0.25 s, for a player's missile (FUN_00112800, read as
  "a human player's": medium confidence) with a target or a side other than NEUTRAL
  (0x22731c-0x22732d), a hostile (only its target, if it has one) within Damage radius x 0.375
  = 1.69 m of it, to their root, that it has passed. It runs after the give-up and the timers.
  The parent then splits; a child (only while it holds the split's target) goes off.
* **The heading stays the launch's** (a child's, the split's). The 60-degree test and the
  seek's cone read the node's -z (FUN_0001fdd0 at 0x226fdc), and each physics step
  (FUN_00169e20 -> 0x159eb0) sets the node back from the missile's body, whose turn is set only
  at the launch and the split (FUN_00227ce0's vtable +0x50): FUN_002266d0's turn of the node
  (0x2268b4-0x22690d) shows only in that frame's picture. So the drawn missile faces its
  velocity, and the tests read the direction it left in. Round 3's one-step lag is gone.
* **Hitting something** (the collision callback FUN_002278e0): the map's collision, a living body
  (not the shooter's) standing (play.rs's cylinder) or lying knocked down (its ragdoll's bones,
  each a ball 0.2 m round: `LYING_RADIUS`, the demo's shape; the game's missile strikes the
  ragdoll's bodies, where the plain shots pass over them), a breakable object, or a Sentry. A
  body struck takes a roll of Damage min..max (84-96, FUN_0022c4d0), x the hit-location factor,
  x 0.7 on the shooter's side (FUN_0004b9c0), x the victim's difficulty step and damage-type
  factor. **The hit-location factor** (FUN_00125940 at 0x227a03, the bone from FUN_00115000;
  the missile's own +0x1cc is 1.0): the victim type's `damage-multiplier` (h_e7e9fcd3: 0.62 for
  the campaign squad, 1 for the Shrikes, and 1 for the sdm_bf squad of the squad deathmatch
  maps, which `Game::load_squad_library` reads with its skill data) x the struck hit part's
  bone's `Damage` from its motion set's `<collision>` list (the head x5, x3 for Brutus and the
  hounds; upper arms x0.8; forearms and calves x0.5; the hips, spine and thighs x1; no feet).
  The struck part is the first of the body's hit-part surfaces along the missile's way, from
  0.5 m before the contact (`STRUCK_BACK`; the ragdoll's nearest bone while knocked down): see
  **Damage** below, where the same factor, the step and the log are shared with every gun's
  shots. Blast shares take none of it (FUN_00224a90 passes 0). A breakable object struck takes
  a roll too (the scenery applies its damage-type factor), then the blast's share. A missile that hits something before its split goes off there
  without splitting (the takes' gate, the Shrike at 7 m). A child whose 5 s run out goes off where
  it is (h_0976a9dc, FUN_00227760).
* **The blast** (FUN_0022bfc0, dealt once by FUN_00224770) goes off at the missile's centre:
  the bullet's radius (0.1 m) off the surface it struck, along its normal. Its own roll,
  0.05-0.25 s later (at random), to every living body it reaches with nothing of the world
  between the blast (0.25 m off the surface) and the body's middle; a character behind cover
  isn't gathered at all (FUN_00224a90), so it takes nothing and isn't pushed (take22). With
  Damage h_011cb154 false (the Therm Sweeper's) that's half the roll at any distance
  (FUN_0022c4d0), x 0.7 on the shooter's side (the shooter too), x the damage-type factor. So a
  body struck directly takes a roll, then half another. A body is reached when its standing
  cylinder, 0.6 m round (`BLAST_BODY_REACH`), is within the Damage radius (4.5 m): a fit, not
  traced (the game gathers the character's collision bones, each with its own radius): Tex was
  hurt by his own blast on a wall 5 m from his feet and not at 6 m (takes 12, 13, 15), which
  play.rs's 0.4 m body missed at 5 m. Loose pickups are thrown. The scenery takes the same
  share, flat (`ObjectBlast::flat`): the blast's entry for an object other than a character is
  its own centre (FUN_00224a90), so FUN_0022c4d0 sees it at distance 0: half the roll here
  (h_011cb154 false), the whole roll when true, no falloff. (The grenades' scenery keeps its
  falloff: their path isn't checked against this one.)
* **No knockdown** (`hurt_still`): in the takes no hit or blast of a missile knocks its victim
  down or shoves it (Tex hurt by his own blasts, takes 10-16; frozen enemies hit directly and by
  blasts, takes 06, 18, 19, 23), where play.rs's `hurt` floors a body for 15 HP or more two
  times in three. The game's knockdown rule isn't traced. A kill still throws the body (the
  death push). An enemy of a type of its own reacts by its personality whatever hurt it
  ([enemies.md](enemies.md), "Hit reactions").
* **The explosion** (FUN_0022cfc0): the bullet's effect type h_173132f0 (exp-rocket-dirt, -fire,
  -fire#1, -shrap, -flash, -fire-add, light_explosion and the sound 0ab060f5) at the blast
  point, turned with its -z along the missile's way (FUN_0022cbe0's FUN_0001d0c0 frame: y up for
  a level shot); a timeout's upright (FUN_0022cea0). It's compiled with the grenades' recorded
  rules (exp-rocket-* are exp-lrg-*'s siblings) and two of the game's own, for the Therm
  Sweeper's trail and explosion only (`AleAssets::load_recorded_facing`):
  * **Facing 3** (`FACING`, `src/ale_fx.rs`; also the muzzle effects', above):
    exp-rocket-add.app and exp-rocket-shrap.rect.app are AngleFadeOut ("perp") appearances with
    GeneralApp_FacingType (e3542300) 3: a quad stretched along its particle's motion (+z of its
    frame for one born still) and turned about it to face the camera, as the particle vertex
    shader (0x3b8bf8) builds it; facings 0, 1 and 2 are the particle's transform, the camera
    and the plane square to its motion. The takes' fireball is upright rectangles facing the
    camera, where lying across y they drew long horizontal streaks edge-on. The flag the demo
    calls PERP (0d645074) is GeneralApp_AngleFadeOut: the shader fades a quad seen within 20
    degrees of edge-on (not drawn: the demo's materials are shared per life step).
  * **Shapes** (`SHAPE`, `src/ale_fx.rs`): GeneralApp_ShapeType (17d45550) picks the particle's
    vertex list (the table at 0x402a4c): 2, a quad from -1 to +1 (twice its Scale across), 3, a
    quad standing on the particle, 5, a quad from -0.5 to 0.5. The demo drew every quad as a 5;
    these effects draw 2 twice as big and 3 standing on its particle. The other effects keep
    the half-size quads (411 of the library's appearances are shape 2): their own check.
  * The Frag's exp-lrg-add.app and exp-lrg-shrap.rect.app are facing 3 too: the grenades keep
    their checked rules (perp quads facing out, `PERP_RADIAL_SPEED`) until they're checked
    against the Frag recording.
  * light_explosion lights the scene as before. The effect types with a light-effect also list
    a light object type in h_f8c6a8b4 (h_09020c30, a point light of range 10, for the Therm
    Sweeper's muzzle and explosion); that the light effect drives it is an inference from the
    data.
  * **The count and the distance LOD** (`Compiled::counted`, `game_count`, `LOD_TAN` in
    `src/ale_fx.rs`), for the Therm Sweeper's trail, explosion and launch dust:
    * Each effect object has a distance LOD from the camera (the render cull FUN_000e7850 ->
      FUN_00156050): d = tan(35 degrees: the game camera's 70-degree view) x the distance past
      the effect type's bounding sphere (FUN_00219ed0: centre h_e99750a9 in the effect's
      frame, radius h_1ab09530; the explosion's (-1.99, -3.21, -1.67), 25 m; the trail's
      radius 0; the dust's (0.08, 0.19, 4.13), 5.21 m; `EffectType::bounds`); LOD 1 to 10 m,
      falling to 0 at 60 m (0x2194d0, 0x2194e0; x (2 - w) / 2 for the world's load and x 1 for
      one view, both 1 in the demo).
    * Each emitter's new particles (FUN_00065800), once a game frame (30 a second,
      `GAME_FRAME`) whatever the frame rate: the whole number of its frequency's over the time
      since its last batch (the frequency read at its end), its initial particles on its first
      update, plus each emit count key passed (cut to a whole number each time), then x its
      Emitter_LODCurve at that LOD (fe077e05, 0x659ed-0x65a12), cut again; the fractions are
      thrown away. Cut each update, the count depends on the step: round 3 stepped it once a
      render frame, and exp-rocket-fire made 1 sprite at 15 and 60 fps where 30 gives 2. A game
      frame's batch is born at its end, spread over its part of the effect's move, and starts
      back by its age along its way (`animate` moves every particle by the whole render frame).
      So exp-rocket-flash makes 3 particles (2 at 0.076 s, 1 at 0.16 s) up to ~72 m and none
      past ~94 m; the launch dust's 18 a second for 0.43 s make one card, at ~0.13 s. The
      initial particles come once (the flag the emitter's constructor FUN_00065640 sets). A
      trail emitter starting its 4 s over (a child's flight past 4 s) counts from 0 again, the
      demo's choice (the game's loop of an emitter's time isn't traced; it may stop it at its
      lifespan): its rate starts over at 0 for 0.07-0.09 s, so the trail has a ~0.2 s gap there.
    * A missile's trail is placed from the missile's own transform each frame, so each game
      frame's particles are born along the missile's move up to where it is then; read from the
      last frame's placement, they were born a frame behind it (2 m at 15 fps). The game's:
      its world update runs the physics step, which moves the missile (FUN_00169e20 at
      0xd9ad4), before the objects' ticks (FUN_000d9840 at 0xd9d0b), where the trail's
      (0x21b7a0) follows the missile and births along its move since the last frame. Like
      every effect on a parent just made, it starts on the frame after the missile's, when the
      missile starts to move, so its time keeps with the flight.
    * A particle's size is its Scale x its Appearance_LODCurve at that LOD (0c53b008; 1 on these
      effects) x the shape.
    * Every other effect keeps the demo's count (a rate's fraction carried over, an emit
      count's burst at the start) and no LOD: the general rule is a ticket of its own.
* **The scorch** (bullet h_06a27365 = h_e6868a8d: 5 x 5, RGBA 10 10 10 200, 25 s, fading over the
  last 5), from the code: the damage handler (0x22d0b0) lays a hit's decal (FUN_0022ca20) on a
  direct hit only when the Damage radius is 0.05 m or less (bullet holes). A blast weapon's
  comes from its blast's pass over what it reaches (FUN_00224770 back into the handler), whose
  entry for the static world sits at the blast's centre with its normal straight up
  (FUN_00224a90): the decal is laid round the blast point and projected down, on the ground
  under it, after a wall hit, a body hit (take03: under the enemy struck, from f256), a timeout
  or the fuse alike. The demo lays its flat decal on the ground below the blast point if that's
  within the radius (the projection's reach isn't traced), its size read as half sizes, a 10 m
  quad as the grenades' (the takes' pair on the cliff, ~10 x 5 m). The pass comes with the
  blast's damage, 0.05-0.25 s after the explosion; the demo lays it 0.7 s after (`DECAL_DELAY`,
  the grenades'): laid that early, the dark decal shows through the demo's paler added fireball
  as a hard dark disc (#104). The takes show it from ~0.85-1.0 s, once their fireball clears.
  At take04's cliff foot it showed only as a straight-edged wedge: the terrain's layers were drawn
  over it (see the terrain layers below); now it's a dark blot there, with the flat quad's
  straight edge where it leaves the curved rock.
* **The camera shake** (`play_shake.rs`): every missile blast shakes the player's view
  (FUN_00228c70, whose only callers are the missile's two explosion paths, 0x227867 and
  0x227b1d: a grenade's blast doesn't shake the view through it). A camera within 7 x the Damage
  radius (31.5 m) gets clamp(Damage max x 0.00769, 0, 1) = 0.74 x f, f being 1 within half that
  reach and falling to 0 at its edge (nothing when f is 0.1 or less there), held 0.5 s, then
  falling to nothing over 0.75 f s (FUN_0006bdd0). Each game frame the camera's update (0x6c780:
  FUN_00073d20, FUN_00073c00, FUN_00073a80) turns the view by a random pitch and yaw within
  +-0.025 x the amplitude rad and a roll within half that (FUN_00072010, FUN_000720d0,
  FUN_00073d80), new each frame. The view only, the demo's choice: the aim keeps the unshaken
  camera. The game multiplies the turn into the camera's own frame (0x6c54a, FUN_00073b20, the
  camera's vtable +0x3c); whether its crosshair ray reads that frame isn't traced. A weaker kick
  doesn't replace a stronger one still running.
* **The terrain's layers sort first** (`TERRAIN_LAYER_SORT`, `src/level_scene.rs`; see
  [levels.md](levels.md)). Bevy sorts transparent meshes by their origin's depth, and a terrain
  layer's origin is the world's: where that origin was nearer the camera than an effect, the
  layer was drawn after it and painted the ground over it. At take04's cliff the blasts and the
  launch glow vanished that way (the tester's "no blast pixels over 200"), and the scorch showed
  only where the rock layer's mask let it through.
* **The difficulty step** (#127, see **Damage** below): every hit in the takes' HP logs is x1.4
  the code's rule on enemies and x0.6 on the squad, the character damage handler's (0x125600)
  1 - 0.1n and 1 + 0.1n on STANDARD (n = -4). The direct hits and the blasts here take it: on
  STANDARD, the demo's default on the missions and the test map (the multiplayer maps have
  none), Tex's own blast is 0.5 x 0.7 x 0.6 = 0.21 of a roll (takes
  10-16: 18.2-20.1), take03's direct x1.4 of a roll (118-134). The numbers under **Verified**
  below are #73's, from before the step (x1 there: 32.8 for Tex's own blast, 93.6 and 42.7 for
  take03's geometry) and before the hit-part surfaces (the bones the homing takes' children
  struck were then the nearest to the contact); **Damage** has the checks since.
* **Not done**: the AI's 75 m noise (FUN_0004d490), the damage event (FUN_001fe3d0), the second
  sound list h_03dc7b9a, the red enemy crosshair (a shared HUD feature).

Test hooks: `BF_MISSILE_LOG=1` prints each missile's launch, seeks, retargets, split (children,
targets), fuse and end (where, how far, how long after the launch, what it struck), each direct
hit's amount with its roll, hit-location factor, struck bone and step (`x0.31 at bone
h_044f3505, x0.60 the step`), and each body its blast reaches (with its step) or finds behind
cover (`BF_COMBAT_LOG` adds each hit's factors, see **Damage**); a give-up says how far and how far off
its heading the aim point was. `BF_MISSILE_DEBUG=1` also prints every body each target search
looks at, with the root it measures to; `BF_SHAKE_LOG=1` prints each shake kick and its end.
The takes' places, with
`BF_TEST_WEAPON="Therm Sweeper"` and taps as `BF_TEST_TRIGGER=3.0-3.15,...`:
* take01 `BF_MAP=e34 BF_TEST_GOTO=-53.64,31.07,-53.76,31.56`; take04 `BF_MAP=e40
  BF_TEST_GOTO=-16.00,152.32,-16.50,152.26` (the take's view, the impact on the reticle:
  `BF_CAMERA_PITCH=-0.10 BF_TEST_FREECAM=0.5,-13.24,28.9,152.81,80,-7.5`); take06 `-- --test
  BF_MAP=e40
  BF_TEST_GOTO=17.15,175.43,16.69,175.23 BF_CAMERA_PITCH=-0.2 BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC=0.5,brutus,enemy,dummy,-5.81,165.55` (the crosshair on him; the take's view:
  `BF_TEST_FREECAM=0.5,19.72,31.14,176.54,66.7,-15`).
* The gate takes (10-16, 22, 23): `-- --test BF_MAP=e34 BF_TEST_GOTO=-55.2,z,-55.2,z+0.5` with
  Tex d m from the panel (its surface at z 41.285, so z = 41.285 - d), and Flint or an enemy put
  on the floor under y 31.5 (`BF_TEST_NPC`'s height, or it lands on the gate's top): take14
  `BF_TEST_NPC=0.5,flint,friend,dummy,-54.2,39.285,31.5`, take16 at x -56.2, take22's enemy
  behind the panel at (-56.5, 42.8) (the take's (-55.2, 42.8) is within 5 m of the gate's door,
  which opens for anyone in the demo, enemies too), take23's in front at (-56.7, 40.5).
* The homing takes (17-19) on the test map: `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_CAMERA_PITCH=-0.08 BF_TEST_INSTANT_KILL=0` and Brutus dummies 24 m out 2 m either side of
  the line (17), 30 m out (18), or 30 m out 4 m left (19); the takes' camera 3.5 m behind Tex:
  `BF_TEST_FREECAM=0.5,4.5,1.3,23.5,0,-3`. take07's 45 m: one dummy at (4.5, -25), with
  `BF_CAMERA_PITCH=-0.115` to put the crosshair on it (at -0.08 the line passes over him). Four shots
  2.4 s apart, each at fresh dummies (`BF_TEST_TRIGGER=3.0-3.15,5.4-5.55,7.8-7.95,10.2-10.35`,
  the dummies spawned again at 5.0, 7.4 and 9.8 s, and
  `BF_TEST_NPC_REMOVE=4.95,all;7.35,all;9.75,all`) give eight children each.
* A side view of a launch (the dust): the same spot, `BF_TEST_FREECAM=0.5,10.5,1.6,19.0,90,-5`.
* The reload (08): `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5 BF_TEST_RESERVE=32 BF_AMMO_LOG=1
  BF_START_WEAPON=1 BF_TEST_TRIGGER=2.5-11.9,15.5-15.65,...`; the open air (20): the same spot,
  `BF_CAMERA_PITCH=0.17`, one tap.

Verified (15 fps captures and the logs; the takes' damage x1.4 or x0.6, see the difficulty
step):
* Timing: the missile leaves on the fire loop's frame with the clip drop, ed761bae and 0021996c
  (`BF_SOUND_LOG`); the split comes 20.0 m and 0.667 s after the launch every time. take04's
  cliff: the children strike it 23.4 and 24.3 m out, 0.77 and 0.80 s after the launch (the
  takes: blasts 0.80 s after the glow at ~25 m).
* Open air (take20): the children's timers run out 5.67 s after the split, 168.8 m beyond it,
  and they go off where they are (the take: 5.58 s game time, 168.6 m, a climb with no drop).
* Homing (round 5: the root, the fixed heading, the give-up before the fuse; the dummies are
  Brutus, damage-multiplier 0.62 and holding his gun in front, where the takes' Shrikes are 1;
  four shots each, `BF_MISSILE_LOG`):
  * take17 (two enemies 4 m apart, 24 m out): the split, 2.9 m short of their hips and ~38
    degrees off, deals neither out in any of the four; the children fly on between them, nobody
    hurt, as the take shows.
  * take18 (30 m out): the split gives each child an enemy; all eight strike, at a forearm
    (x0.5, so x0.31) but one at a thigh (x1, x0.62), then each enemy takes both blasts' shares
    (42-47): a x0.5 direct and both shares, as the analyst reads the take.
  * take19 (one enemy 30 m out, 4 m left of the line): 5 of the 8 children strike him, each at a
    forearm (x0.31); 3 give their aim up 4-6 m short at 61-62 degrees off their heading and fly
    on, landing 36-42 m out. The take: all four children of its two shots struck, at x1 parts.
    Not matched (see below).
  * take07's 45 m (the enemy on the line): the split, 25 m short, finds nobody; the children,
    turned 9 degrees and homing on the parent's point 200 m ahead, fly parallel 1.05 m either
    side of his hips (the demo's body is 0.4 m round) and on: no hit, no share. take07's two
    children struck him directly (-256.8 at once, two x1 directs): not matched (see below).
  * take06 (25 m, three shots 2.37 s apart, the tester's setup): the children give their aim up
    within 1 m of his hips at the end of their straight phase (0.95 m, 93-95 degrees off) and
    fly on: shot 1, one share (46.5, the blast 4.3 m from him; the other child lands 26 m on),
    shot 2 one (44.8 at 1.3 m), shot 3 none (the nearer lands 6.1 m from him). take06: one share
    per shot (61.6, 64.6, 64.0), both on shot 4. Round 4 gave two on shot 1.
* On sdm_e34 (the sdm_bf squad, damage-multiplier 1): a Brutus dummy 7 m ahead of the first
  start point (`BF_MAP=sdm_e34 -- --test BF_TEST_NPC=0.5,brutus,enemy,dummy,-0.26,41.69`, a tap
  at 3 s) takes the missile on his thigh, x1.00 (93.6 of a 93.6 roll), where the campaign's types
  give x0.62.
* The count once a game frame (round 4): take18's two blasts emit the same at 15, 30, 60 and
  144 fps (`BF_CAPTURE_FPS`, `BF_ALE_LOG=1` prints each batch): per blast 3 flash, 2 + 2 fire,
  5 fire-add, 6 dirt and 43 shrapnel particles, the analyst's simulation of FUN_00065800. The
  trails differ by a game frame or two at each missile's end (see below).
* Self-damage (takes 10-16, the gate): Tex 2, 3, 4 and 5 m from the panel takes 32.8 from his
  own blast (half of 93.6, x 0.7), at 6 m nothing (the takes: 18.6-20.1 at 2-5 m, x0.6; nothing
  at 6 m); he isn't knocked down. Flint 1 m to his left or right takes the same 32.8 (takes 14,
  16: the same for both, 19.3 and 18.7).
* Cover (takes 22, 23): an enemy 1.5 m behind the gate's panel is never reached by three blasts
  on it (behind cover from the blast's point); one in front of it, right of the hits (take23's
  spot), takes a half roll from each (46.8, 42.5; take23: 65.9, 62.0, 65.3, x1.4) and isn't
  moved.
* take03's geometry (an enemy 7 m out): 93.6, then 42.7 0.13 s later (take03: 128.8, then 64.9
  0.2 s later). A friend on the line: 93.6 x 0.7 = 65.5.
* Reload (take08, the gun in Tex's second slot): the held trigger fires every 1.67 s; the
  reload starts at the release, the reserve dropping 32 -> 26 at once, and the clip refills
  1.13-1.2 s later at Tex's Sc_w2_reload magazine-in event (1.20 s; FUN_00117660, event
  1b2ec99e moves the pending rounds into the clip); the take: 1.26 s held, 1.15 s after taps
  (Tex's Sc_w1_reload has it at 1.00 s: `BF_TEST_WEAPON` alone puts the gun in his first slot).
  After the last tap the reload starts 0.13 s later (the take: 0.19 s). So reload-time 5 isn't
  a 5 s reload.
* Camera shake (`BF_SHAKE_LOG`): kicks of 0.264 (take06) and 0.225 (take04) for blasts ~26-27 m
  from the camera, over 0.77 and 0.73 s: a turn of up to +-0.0066 rad, which shows as a jitter
  of the frames. The takes: +-1-5 of 960 units for ~0.7 s at 25 m (take04), +-5-10 for ~1 s at
  6 m (take03, where the code gives the full 0.74).
* The look, from the takes' own views (`BF_TEST_FREECAM` above), in 640 x 480 units² above
  luminance 200 (and 240):
  * take04 (the cliff, 25 m), with the impact on the reticle (round 4's view cut the fireball's
    top off, the impact ~50 units above the reticle, and read 17.8k): the two fireballs peak at
    25.4k (18.3k), box 227 x 162 units and reaching the HUD band, bright for ~0.5 s (frames
    54-61); the take at 16.9k (12.7k), 216 x 125, ~0.47 s. **Not matching: 1.5x**, taller and
    whiter. The launch glow lights ~14.0k units² by 40 or more over the frame before (the take
    ~19.2k), 0.2-0.27 s after the shot (the take's 0.3 s): short, and not from the trail's birth
    position (the analyst's Q1). Before the terrain-layer fix neither showed at the cliff.
  * take06 (the sand, 25 m), one fireball at a time: the first shot's nearer blast peaks at
    10.3k, box 137 x 115, bright ~0.4 s; the take's first (f250-f278) 6.5k, 154 x 103, 0.47 s or
    more: 1.6x, the same excess as take04's. The launch glow up to 5.6k (the take up to 4.7k).
  * take20 (the open air, ~190 m, LOD 0): no white flash; each child's blast makes 1 fire-add
    sprite, 1 dirt card and 3 sparks (the analyst's counts), an orange flame ~30 x 25 units for
    ~0.4 s from the takes' camera (the take: 12 x 15 units, ~0.33 s, no flash); round 2's
    ~55-unit white disc is gone.
  * The launch dust: one brown card rising behind the launcher from ~0.13 s, out of the follow
    camera's view (the side view above). From the game's camera aimed (#126's leads: the aim
    shift brings Tex's 3.9 m boom to ~2.2 m behind his head while the gun is up;
    `BF_TEST_FREECAM=0.5,4.5,0.9,22.2,0,-3`) the card is born behind the camera: no veil (the
    frame's mean rises by 2.6 at most, the launch's light). From the 3.5 m spot of the takes'
    view above it's born just in front of the camera and veils the view for ~0.5 s (mean red
    48 -> 91 -> 50). Which of the two the takes' launches are seen from is #126's to settle.
* Muzzle effects (above), the branch against main in the same side views (`-- --test
  BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12 BF_TEST_FIRE=1 BF_CAMERA_DISTANCE=2.2
  BF_VIEW_YAW=<yaw>`): the sphere emitters' glows (A10, Saryl-45, Jax-iC) are round as on main;
  the Minigun's flash leaves the barrel forward (main's fires across it); the scoped MK draws as
  on main.
* The main game on sdm_e34 (`BF_AUTOPILOT=1`, 300 frames) differs from main's by at most 0.4%
  of the pixels from frame 8 to the first shot (296; #120's stamina bar; frames 1-7 differ while
  the scene's render pipelines finish compiling). From the shot on, Tex's LZR-23 plays its
  muzzle effect type's light (light_muz_laser): a reddish light, as the LZR-23's take01 shows on
  the ground at Hawk's feet (f196-f200), where main's demo light was orange-white and brighter.
  Not measured.

Not matching or not done:
* The explosion is too big: 1.5x the take's bright area on take04's cliff (two fireballs) and
  1.6x on take06's sand (one), taller and whiter. Nothing in the game's explosion depends on
  the surface (the analyst's Q2: placement, frame, slots and LOD are the same on both), and no
  exp-rocket appearance uses shape 3 (Q4). Left for a ticket of its own; its leads: the angle
  fade of the facing-3 quads near edge-on (exp-rocket-add's, up to 13.6 m), which the demo
  doesn't do, and the added sprites summed in linear light (#104). take20's open-air flame is
  ~2x the take's. take04's late fire column (the take's at +1.4 s) is missing. The curves'
  easing (the demo reads easing bytes 0, 5 and 6 as linear, `bf/ale.rs`'s ease; no exp-rocket
  curve uses them) stays open.
* The launch glow is ~0.7x the take's (14.0k against 19.2k): not the trail's birth position
  (Q1); the trail head's look or the muzzle light (light_muzzle), not traced.
* A missile's trail stops with the missile: its entity goes in the frame the missile ends,
  before the effects run, so the game frames of that render frame aren't emitted (up to 2 at
  15 fps, 1 at 60 fps). take18's three trails make 11, 25 and 8 particles (mtrail, mtrail#1,
  the flare) at 15 fps, 15, 31 and 8 at 30 and 60 fps, 15, 32 and 9 at 144 fps.
* The scorch is a flat quad: at the cliff's foot it has a straight edge where it leaves the
  rock, where the game's wraps the rock (its decal manager clips the world's triangles,
  FUN_00214710 -> FUN_00217920). On flat ground it's the takes' dark ragged blot.
* The children's hits on take19 and take07: they home on the hips now (the analyst's Q3), and
  take06's one share per shot holds on two of three shots, but take19's children strike
  Brutus's forearms (x0.5, his gun held in front) where the take's struck the Shrikes' x1 parts,
  and 3 of 8 give up at 61-62 degrees off their fixed heading 4-6 m short and miss, where the
  take's four all struck. take07's children pass 1.05 m either side of him (the split's 9-degree
  turn and the straight phase's drift, then homing on the point 200 m ahead) where the take's
  both struck him. The demo has no Shrike to test on, and its body is a 0.4 m cylinder where the
  game's are the collision parts; which of those (or the paths) makes the difference isn't
  settled.
* The demo's follow camera (its 45-degree view and the aiming dolly, #126) puts Tex over the
  crosshair, so the launch and the blasts are mostly behind him. On e40 Tex and the standing
  stones draw black (main too): not this ticket's.
* The other effects keep the demo's quads, perp rules and count, and have no LOD: the shape 2
  appearances elsewhere (half the game's size), facing 3 on non-perp appearances (the Psionic
  Blast's muzzle ball), the Frag's facing 3 quads. Hit lights (an effect type's light on a
  bullet's hit) aren't played.

Sounds (`src/bf/audio.rs`) come straight from the game's banks: `sounds-<level>.xmb` maps sound
ids to Xbox ADPCM data in `sounds-<level>.mem`, decoded to 22 kHz WAV in memory. Each character's
archetype names its jump grunt and footstep type (Tex 1, Flint 2, Brutus 3, Hawk 4), and
`world-materials.xmb` gives every surface four footstep variants per type plus landing and slide
sounds. Footsteps fire when a toe comes down into the lowest quarter of its swing in the clip
being played. Surface sounds live in level archives: `BF_LEVEL=<name>` (default `e01`) picks the
level; `mp1`'s bank is merged in too, since a mission only carries its own squad's footsteps.
On a level, its sound-triggers play their own voices (#223: voice lines, door and gate sounds,
ambience, the music cues; 3D ones by DirectSound's distance law from the camera): see
[triggers.md](triggers.md), "Sound- and dialog-triggers", and their `BF_SOUND_LOG` lines
("trigger voice ...").

Test hooks: `BF_AUTOPILOT=1` plays a scripted run (idle, run, turn, sprint, aimed walk,
backpedal, dodge, standing jump, running jump, aimed fire, weapon switch, firing on the run); `BF_TEST_EXPLOSION=1` sets off the selected grenade type 6 m ahead every 1.5 s; with `BF_TEST_GOTO`, `BF_TEST_THROW=<s>[,<hold s>,...]` holds the grenade button (G) at those times (see **Grenades** for the rest of its hooks); `BF_ANIM_PROBE=<script>,...` prints each character's clips by script name (duration, root motion over a cycle, the clip's own speed h_f9628571, events); `BF_DUMP_TEXTURE=<file>:<hex id>` writes a decoded texture (raw RGBA after a u32 width and height). `BF_AUTOPILOT_SPIN=<rad/s>` makes it look down and turn the camera during the standing fire; with `BF_CAPTURE=<dir>` it saves every frame at a
fixed 15 fps. `BF_MUTE=1` silences audio, `BF_SOUND_LOG=1` prints every sound event with its time
and action, `BF_DUMP_SOUND_IDS=<dir>:<id,id..|all>` decodes sound ids to WAV, `BF_DUMP_SOUNDS=<dir>` writes the character's jump and surface sounds as WAV files
and exits. `BF_DUMP_WEAPONS=1` lists every character's weapons as loaded (definition, model parts,
hardpoints, fire sounds) and exits; `BF_DUMP_TYPES=1` (or `all`: every archive) lists the
character types, their motion sets' clips and the placements, and exits
([character-types.md](character-types.md)); `BF_SHOT_LOG=1` prints each shot's barrel and shot
directions. Camera hooks: `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` / `BF_CAMERA_DISTANCE` set the
start (with `BF_TEST_GOTO`, `BF_CAMERA_PITCH` holds the pitch throughout, e.g. to fire down at
something), `BF_VIEW_YAW` turns only the rendered view (aim unchanged), `BF_START_WEAPON=1` starts with
the second weapon. Captures wait 60 frames for shaders to compile before the clock starts.

### Damage: the difficulty step and the hit parts (#127)

`src/bin/play_damage.rs`, from default.xbe's damage path as the analyst read it
(`todo/damage-factor/answer.md`; every function cited is in [code-map.md](code-map.md)). A hit
on a character goes:

* **The roll** (FUN_0022c940): Damage min + U x (max - min), x a character shooter's skill's
  factor (its +0xec at 0x22c9a3; `Hit::skill`, the log's "x the shooter's skill"): Flint's x2 on
  15% of her rolls while her targeting is on and x1.5 scoped (0x13cc70, [skills.md](skills.md#flints-advanced-targeting-124));
  1 for the others. Every roll a shooter makes takes it: a shot's on a body or a breakable
  object, a missile's direct hit and its blast's own roll. The damage power-up's char+0x754
  isn't in.
* **The hit record** (FUN_0022c4d0): a direct hit takes the roll, a blast its share of the
  blast's roll (half for h_011cb154 false, the Therm Sweeper's); then x the hit factor: an
  instant ray's range falloff (FUN_00230040, **Shots**) x FUN_00125940; a projectile's
  FUN_00125940 (the missile's +0x1cc is always 1.0, FUN_002278e0); a blast's share 1
  (FUN_00224a90 passes 0 to FUN_00224320). **FUN_00125940** is the victim type's
  `damage-multiplier` (h_e7e9fcd3, type+0x220: the campaign squad 0.62; the Shrikes and the
  squad deathmatch's sdm_bf squad 1) x the struck hit part's bone's `Damage` (the motion set's
  `<collision>` list: the head x5, x3 for Brutus and the hounds; upper arms x0.8; forearms and
  calves x0.5; Bip01, Spine2 and the thighs x1). Then x0.7 when the shooter and the victim are
  both the squad's (FUN_0004b9c0; the demo's sides stand for its squads: the same team, the
  shooter's own blast included).
* **The victim** (0x125600, a character's combat-target vtable slot 0): x (1 - 0.1n) on an
  enemy team, x (1 + 0.1n) on the squad's, n the difficulty; then its skill's factor (Vengar's
  0.85 while on, `Hit::victim_skill`, skills.md); then FUN_002232d0's factor for the damage type. Every hit on a character
  reaches it: the guns' hits and blasts, the scenery's damage areas (FUN_0021ade0), falls
  (FUN_0012b740, DTYPE_NORMAL), FUN_00223160 (Brutus's charge). Only the trainer's `!hit` skips
  it (it calls FUN_002232d0 itself), so older xemu logs of `!hit` carry no step.

The victim's values are its character type's (`Player::kind`, #128,
[character-types.md](character-types.md)): its hit parts (its motion set's `<collision>` list)
and its damage-type factors by type; its damage-multiplier by its squad slot's own entry first
(the sdm_bf squad's 1 on a squad deathmatch map), else its type's. The squad's are what they
were; a test map NPC of an enemy type takes its own (a Shrike's SONIC x2 and GAS x0.2, its head
x5).

**The difficulty** (`Difficulty`, set as each map begins):

| Menu (FUN_000f13c0) | `<difficulty-settings>` (game-options-en) | n | Enemies take | The squad takes |
|---|---|---|---|---|
| STANDARD | `easy` -4 | -4 | x1.4 | x0.6 |
| HARD | `normal` 0 | 0 | x1 | x1 |
| BRUTAL | `hard` 4 | 4 | x0.6 | x1.4 |

* The values are read from `common/game-options-en.xmb` (`Game::difficulty_settings`). The
  sign goes by the victim's team: its row of the team table (0x3ffd80) has 1s in columns 3-6
  for an enemy (teams 1, 7, 8); the squad's 0 and the others take the plus.
* **The default**: the missions and the test map play STANDARD, the takes' (both sides of their
  HP logs fit n = -4: answer.md; #129's campaign takes read n = -4 live). The multiplayer maps
  (the game's level list: the squad deathmatch maps sdm_*, the arenas mp*) play none, n = 0.
  **Read live**: in #129's deathmatch takes on mp1 (todo/129-shrikes/notes.md, B2: takes 35, 36,
  38, 39) the getter gave n = 0 at 0x12566d, and each hit was the roll x the bone's `Damage` with
  no x1.4 (Soldier 30.1-31.4, Heavy 29.6-31.9). Two takes agree: #85's take29 on sdm_e34 had
  Tex take 39.4 of a barrel's 40 (the campaign's took ~24, takes 01-04), and #146's takes 01-03
  on sdm_e13 had Brutus lose exactly 30.00 per lava entry, the lava's rate.
* **How a match gets n = 0 isn't traced** (a note): 0x125600 applies the step on every map, with
  no mode check. The getter (0x7c720) reads the flag from the current slot's tree (the session's
  vtable +0x210, 0x78d60: [session + 0x5cc + 4 x [session + 0x5c8]]) through FUN_00078320,
  which gives 0 only without a tree or without the flag; the default game flags hold it at -4
  (`common/default-gameflags.xml`'s `<campaign>`, where the decoder names its hash h_1a98de0a
  `shadoon-look`, a name collision with `difficulty-setting`). The setter's callers are SELECT
  DIFFICULTY, RESUME LAST and QUICK CAMPAIGN (STANDARD: FUN_000f7140, FUN_000f7230) and the
  system-link join at 0x1eef81; none is on a split-screen match's start path. So a match's slot
  evidently lacks the flag; which slot it uses isn't traced. System-link play, whose join calls
  the setter, is untested (the demo gives it n = 0 like any match).
* **What changes on sdm_e34**, where the demo has no step:
  * the grenades' thrower shares are x1 / 0.6 of STANDARD's: the Frag's thrower takes 21.0
    where it takes 12.6 on STANDARD, as the game's half roll x0.7 gives (0.35 of a roll, 22.1
    for the same roll); the Sonic's and the Energy's thrower shares the same;
  * the Gas poisons at 29.5 HP/s (17.7 on STANDARD);
  * #85 take29's barrel deals 40.0 to Tex, the take's 39.4;
  * every gun's hit on a squadmate takes the struck part's `Damage` and x0.7 (sdm_bf's
    damage-multiplier is 1): a head shot x3.5 (Brutus's x2.1), a forearm or calf x0.35; the
    AI's shots on their own side take the x0.7 too;
  * the missiles (#73 had the part and the x0.7 already) are as #73 had them.
* `BF_DIFFICULTY=<standard|hard|brutal>` (any case) or a number n sets it, on any map (on a
  multiplayer map too: a test hook, to put the takes' STANDARD on the sdm_bf squad's
  damage-multiplier 1, the Shrikes').

**The hit parts** (`struck_bone`). On the squad's models exactly the eleven bones of the
`<collision>` list have a collision surface in their part archetype (`<h_f3f067ad
h_051ac63e>`, objects-common.ipn; `Character::part_collision`, in the bone's frame): the bodies
FUN_00114010 makes a character's hit parts of (FUN_00114d00 builds each from the part's node).
A shot that meets a standing body (the demo's 0.4 m cylinder, `ray_body`) is followed through
it, and the first part surface it meets, posed as drawn, is the part struck: the game's ray or
missile strikes those bodies themselves and FUN_00115000 gives the body's bone. A shot through
the cylinder that meets none of them (between the legs, past an arm) takes the part whose bone
comes nearest its way through the body: the demo's (the cylinder still decides what's hit; in
the game that shot would pass). Knocked down, the part whose bone is nearest the contact on the
ragdoll (#73's rule). The surfaces sit on the drawn models (`BF_HIT_PARTS`): a character
holding its gun in front takes a frontal shot at the hips on a forearm (x0.5), and Brutus's
head (x3) hangs low in front of his chest.

**The grenades and the scenery.** The grenades keep their own rules (grenades.md; the game's
blast rule is #112's) and take the step and the damage-type factor through the same `Hit`.
Their amounts fitted to recordings on the campaign squad on STANDARD (the thrower's share of a
Frag, Sentry or Roller blast, of the Sonic's and of the Energy's bolt, and the Gas's rate) hold
that x0.6 already: they're divided by it (`footage_squad_step`), so they come out as measured
on STANDARD and scale with the difficulty like any hit (with no step, on the multiplayer maps
or with `BF_DIFFICULTY=hard`, x1 / 0.6 of them: the Frag's thrower 21.0 for a roll of 63,
where the game's half roll x0.7 gives 22.1). Measured on the thrower in its own blast, they hold FUN_0004b9c0's x0.7 too,
so no x0.7 is added (`Hit::source`); the others' rule (Damage max falling to nothing at the
radius) is the demo's, without it. The Gas's 41.3 HP/s on an enemy on STANDARD is the demo's
extrapolation (with the x0.7 taken out too it would be ~59). The scenery's damage areas take
the step: the campaign squad's ~0.58 of a barrel (scenery.md). Not stepped: the test hooks'
kills (`BF_TEST_DIE`, the test map's suicide and instant kill) and a harmful liquid (its game
path isn't traced).

**The log** (`BF_COMBAT_LOG=1`): the map's difficulty as it begins (`difficulty on e34:
STANDARD (n -4): enemies take x1.40, the squad x0.60`), then each hit with every factor, before
`hurt`'s line with the hit points left:
`t 3.60: tex -> tex (NPC 1): roll 93.62 x damage-multiplier 1.00 x Bip01 1 x STANDARD (n -4) enemy 1.40 x type 10 1.00 = 131.07`.
A part found by the demo's own rules is tagged after its `Damage`: `(nearest)` for a shot that
passed the cylinder between the part surfaces, `(nearest, lying)` on a body knocked down; an
untagged part is the first surface the shot met. Damage dealt every frame (the Gas, a damage
area) gets no hit line; the areas' own line shows the step. `BF_HIT_PARTS=1` draws every standing character's hit-part surfaces as lines (the
head red, upper arms yellow, forearms and calves cyan, the x1 parts white).

Verified (15 fps, `BF_COMBAT_LOG`; the takes' numbers and allowed ranges are answer.md's):
* **#73 take03** (Therm Sweeper, a Shrike Heavy 7 m out, damage-multiplier 1): `BF_MAP=sdm_e34
  BF_DIFFICULTY=standard -- --test BF_TEST_NPC="0.5,tex,enemy,dummy,-0.26,41.69;0.5,hawk,enemy,dummy,1.3,41.69"
  BF_TEST_WEAPON="Therm Sweeper" BF_TEST_TRIGGER=3.0-3.15 BF_TEST_INSTANT_KILL=0` (the sdm_bf
  squad's 1 standing in for the Shrike's): the direct hit at Bip01, 93.62 x 1.4 = 131.1 (the
  takes 118.2-134.1, allowed 117.6-134.4); the other enemy's share 42.70 x 1.4 = 59.8 (59.07-66.9);
  the squad near it 17.9 (0.21 of the roll). A Brutus dummy in Tex's place takes the missile on
  his forearm (x0.5: 65.5). BRUTAL: 56.2 direct and 25.6 shares (answer.md: 50.4-57.6,
  25.2-28.8), the squad 41.9 (41.2-47.0). sdm_e34's own default (none): 93.6 and 42.7.
* **#73 takes 10-16** (Tex's own blast): at a test map pillar 2 m ahead (`-- --test
  BF_TEST_GOTO=0,-15.7,0,-16.2 BF_TEST_NPC=0.5,flint,friend,dummy,-1,-15.7 BF_TEST_WEAPON="Therm Sweeper"
  BF_TEST_TRIGGER=3.0-3.15 BF_CAMERA_PITCH=-0.05 BF_TEST_INSTANT_KILL=0`): Tex and Flint 19.66 each, 0.5 x 93.62 x 0.7
  x 0.6 (the takes 18.17-20.11; Flint 19.98, 19.67). (At the takes' e34 gate, #73's recipe
  above, the gate opened in this build with Tex 4.6 m from door 6, which has no panels on e34,
  and the missile flew through it: the pillar stands in.)
* **#35 take27** (Brutus shot by an enemy's Saryl-45 at 13 m): `-- --test BF_CHARACTER=0
  BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,20 BF_TEST_NPC=0.5,brutus,enemy,fight,4.5,7
  BF_TEST_WEAPON=Saryl-45 BF_TEST_INSTANT_KILL=0` (the enemy, a Brutus too, carries the hook's gun): 14 hits of
  4.77-5.69, rolls 12.83-15.30 x 0.62 x Bip01 1 x 0.6 (the take 4.93-5.81, allowed 4.74-5.86).
  `BF_DIFFICULTY=hard`: 7.95-9.15 (answer.md's HARD 7.9-9.8). With Vengar on (`BF_TEST_SKILL=8`,
  #122) each hit x0.85: 4.09-4.87 (mean 4.50; the take 4.43), skills.md.
* **#35 takes 04 / 05** (the hovering Shrike's legs): `BF_MAP=sdm_e34 BF_DIFFICULTY=standard -- --test
  BF_TEST_GOTO=4.77,36.82,4.41,37.17 BF_TEST_NPC=0.5,tex,enemy,dummy,-0.26,41.69 BF_TEST_FIRE=1
  BF_CAMERA_PITCH=-0.28 BF_TEST_INSTANT_KILL=0`: the LZR-50 (`BF_TEST_WEAPON="Confed LZR-50"`) 19 hits, all on a calf
  (x0.5), 2.65-5.22 (the takes 2.6-4.9, allowed 2.45-5.25); the Jax-iC 15.4 and 17.5 on the
  calves, then 32.5 on the knocked-down dummy's right thigh: since #124 the shots go to the
  crosshair's lock, which follows the body down. With m09_a's Shrike Heavy in Tex's place
  (`BF_TEST_NPC=0.5,shrike heavy@m09_a,enemy,dummy,-0.26,41.69`, #128) its SONIC x2 makes the
  Jax-iC's 30.7 and 29.7 on the calves (the takes 30.1-35.2; the second roll 21.2, the takes'
  21.5-25.1), then 70.2 and 68.8 on a thigh (#124's lock aim, as above; before it and #129's
  merge, 30.7 and 34.8); the LZR-50 2.48-5.22.
* **#35 take28** (Flint's L-Shot-75, x1 parts): `BF_MAP=sdm_e34 BF_DIFFICULTY=standard -- --test
  BF_CHARACTER=1 BF_TEST_NPC=0.5,tex,enemy,dummy,-0.26,41.69 BF_TEST_WEAPON=L-Shot-75
  BF_TEST_TRIGGER=3.0-3.1,4.5-4.6,6.0-6.1 BF_TEST_INSTANT_KILL=0`, from the sdm_e34 start (no
  `BF_TEST_GOTO`): 44.7, 45.1, 40.7 on the right thigh (the take 40-44, allowed 38.2-46.6).
  Flint's doubled and scoped rolls (#124): skills.md § How it was checked.
* **#115** (the Bower 20's falloff): Brutus's shell at a campaign Tex 7.7 m out on the test map
  (`-- --test BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_TEST_NPC=0.5,tex,enemy,dummy,5.1,11.5 BF_TEST_WEAPON="Bower 20" BF_TEST_TRIGGER=3.0-3.08
  BF_CAMERA_PITCH=-0.16 BF_TEST_INSTANT_KILL=0`): two pellets, aimed at the crosshair's lock
  since #124: 16.70 x falloff 0.682 x 0.62 x 1.4 = 9.88 (Bip01) and 18.57 x 0.678 x 0.62 x 0.8
  (an upper arm) x 1.4 = 8.74 (before the lock aim, 9.85 on a thigh and 5.40 on a forearm). The
  take's m03_a mutant (BALLISTIC x0.8, multiplier 1) isn't in the demo.
* **The scenery's areas**: campaign e34 at #85 take01's spot (`BF_MAP=e34
  BF_TEST_GOTO=-31.52,16.64,-31.52,16.64 BF_TEST_HIT=2.0`): Tex 115 -> 91.0 over 2.13-2.60 s
  (take01: 91.22 by 2.59 s). take29's spot on sdm_e34 (`BF_MAP=sdm_e34
  BF_TEST_GOTO=-53.96,24.6,-53.96,24.6 BF_TEST_HIT=2.0`): 40.0 (take29: 39.4); the
  start-of-map line there reads `difficulty on sdm_e34: none (n 0)`.
* **The grenades** on the test map (STANDARD): the Frag's thrower takes 12.6 of 115 (as before:
  the thrower's share 21.0 x 0.6); the Sonic's (W2 of grenades.md) 8.0 at 4.1 m (7.8 at 4.2 m
  before); the squadmates it reaches x0.6 of before.
* **The hit parts** (`-- --test BF_TEST_GOTO=4.5,20,4.5,19.5
  BF_TEST_NPC="0.5,brutus,enemy,dummy,4.5,13;0.5,tex,enemy,dummy,6,13;0.5,hawk,enemy,dummy,3,13;0.5,flint,enemy,dummy,1.5,13"
  BF_HIT_PARTS=1 BF_TEST_FREECAM=0.5,3.75,1.2,17.5,0,-5`): the four squad models' parts sit on
  their drawn bodies: the head, the upper arms, the forearms on the guns, the torso, thighs and
  calves.

### Running in the background (#163)

For agents' runs while someone uses the PC (`src/bin/play_background.rs`; tooling, not game
behaviour):

* `BF_BACKGROUND=1` (any value but `0`):
  * No gamepad is read: bevy's `GilrsPlugin` is left out, so no `Gamepad` exists. (Pads only
    drive the front end, `play_menu.rs`; in-game control is keyboard and mouse.)
  * The mouse's motion, button, wheel and pointer-move events are dropped at the start of each
    frame, before bevy_input, the UI or `read_input` see them, and bevy_picking's mouse input
    is off. So the cursor is never grabbed or hidden (the two places that lock it,
    `read_input`'s click and the test tools' Esc, also check the hook). The cursor position
    bevy_winit stores on the window, which the UI's hover reads, isn't touched; the menu only
    acts on a hover in a frame with a pointer-move event.
  * The window opens without taking focus, at the bottom of the z-order behind the other
    windows. bevy 0.16 shows a new window with focus, so on Windows it's created hidden and
    shown with user32 `SetWindowPos(HWND_BOTTOM, SWP_SHOWWINDOW | SWP_NOACTIVATE)`. winit still
    thinks it's hidden: a later change to the window's mode, decorations, resizability, level,
    buttons or hit test, or a maximize / minimize, would hide it for good (the demo makes none).
  * Updates run flat out, as a focused window's do.
  * The keyboard and raw mouse motion are left to the focus: winit only delivers them to a
    focused window. Clicking the window (or its taskbar button) brings it forward as usual.

  Use it with `BF_MUTE=1` and hook-driven runs (`BF_TEST_GOTO`, `BF_CAPTURE`, ...).
* `BF_WINDOW_MONITOR=<n>`: opens the window centred on monitor n (0-based, in the order Windows
  lists them; on this PC 0 is the main monitor and 1 the left one), with or without
  `BF_BACKGROUND`. Unset: where it opened before. Past the last monitor bevy logs "Couldn't get
  monitor selected with: Index(n)" and opens it at the default position.

Without either variable nothing changes. Checked (`BF_MAP=sdm_e34 BF_TEST_GOTO=0,0,0,-20
BF_CAPTURE_FRAMES=60 BF_MUTE=1`): with `BF_BACKGROUND=1`, the foreground window (polled every
0.1 s with `GetForegroundWindow`) stayed the full-screen game in front for the whole run, the
demo's window came up visible at the bottom of the z-order, and the captures render normally
while it's covered. A background run gave the same 60 frames, pixel for pixel, as a run
without the hook (run on a separate, never-shown desktop so it couldn't take focus). Frames 2-5
differ between some runs either way, with or without the hook: the terrain and the weapon
appear a frame earlier or later depending on when their GPU pipelines finish compiling. With
`BF_WINDOW_MONITOR=1` the window opened centred on the left monitor. Not tried with live input
(a pad or mouse in use during a run).
