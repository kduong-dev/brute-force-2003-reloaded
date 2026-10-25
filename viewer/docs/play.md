# Playable demo (`bf_play`)

| Keyboard / mouse (no gamepad for now) | Action |
|---|---|
| WASD | move (relative to the camera); the character turns toward the direction |
| Shift | sprint (the game's dash). `BF_TEST_SPRINT=<s>` sprints toward the goal from that time with `BF_TEST_GOTO` |
| Ctrl | walk |
| Right mouse (hold) | aim: the gun is held raised (its ready timer at 4 s or more), so the camera moves in as after a shot and eases out the game's way after the release ([camera.md](camera.md): a keyboard-and-mouse adaptation, the game has no aim without shooting); the gun is turned onto the crosshair while the legs keep moving; moving away from the aim backpedals. Moving across it (30-150 degrees off the crosshair) side-steps, standing (`Sc_w1/w2_rp_f_side_walk_l/r` up to 100 degrees, `rp_b_side_walk_l/r` past it) or crouched (`cr_f/b_side_walk_l/r`). Those clips twist the upper body about a quarter turn off the root, and their root motion runs straight along it, forward or back, as far as the walks. So the character turns until the gun is on the crosshair and walks where the clip takes it: across the aim, a little forward or back. One clip in the squad's data has no root motion: Tex's `Sc_w1_rp_f_side_walk_l` (his first slot, whatever gun is in it, aimed left; its own speed h_f9628571 is 5.9e-05 m/s where the squad's other side steps carry 1.0-2.1). The game doesn't move a walk by its clip (FUN_0012c6c0 moves it at the type's `speeds` along the stick), so there he steps left as he does right. The demo moves by root motion, so a side step whose own speed is under 0.001 (the game's no-speed threshold for the play rate) takes the root motion of its stance's walk in the same direction, forward or back (`root_stand_ins`; #170: he walked in place before). `BF_TEST_STRAFE=<x>[,<y>]` moves that way (camera-relative) with `BF_TEST_GOTO`; with a goal far ahead (`BF_TEST_GOTO=4.5,20,4.5,-80`) the camera keeps still while he steps. `BF_MOVE_LOG=1` prints the controlled character's position, facing, gait and clip every half second (and the stand-in walk, `root from`). Verified (#170, `BF_MOVE_LOG=1 BF_TEST_AIM=1 BF_TEST_GOTO=4.5,20,4.5,-80 -- --test`, Tex with the Therm Sweeper): aimed left (`BF_TEST_STRAFE=-1,0`) 1.59 m/s and right 1.60, back-left / back-right 1.53 / 1.54, crouched 1.72-1.73 each way; the planted boot holds within 1 px of the floor on both steps; main's build stepped left in place; Hawk's aimed steps and Tex's unaimed walk are pixel-identical to main |
| Left mouse (hold) | fire the held weapon (at its own rate; also aims while held). Standing, the feet turn with the aim (the spine keeps at most ~11° of twist, also when looking down and turning). As in the game, the camera moves in along the crosshair's ray from the first shot (the aim shift: Tex's 3.9 m to 2.2), stays in for the ready timer's 4 s and 2.33 s more, then eases out over 0.67 s; a dash, a jump, a crouch's transition, a throw, a reload or a switch lowers the gun and takes it out sooner ([camera.md](camera.md)) |
| Q | switch between the character's two weapons (animated: stow one, draw the other; plays the game's hard-coded switch sounds ff820fda / e1e97460, 0.65 s apart: the HUD weapon panel's, FUN_000cbec0, played only for a character a player controls, FUN_000bb220, so an AI member's switch, a script's toggle, is silent; the controlled character's is heard in a cutscene too) |
| Space | jump: standing jumps crouch first, running jumps launch straight away and carry the run's momentum (Brutus uses his four-legged jump when sprinting on all fours) |
| C | dodge roll / sidestep to the left or right of the movement. `BF_TEST_DODGE=<s>[,<s>...]` presses it at those times with `BF_TEST_GOTO` (to the right, standing); `BF_TEST_DIVE=<s>` dives toward the goal then, as from a grenade landing behind (a squadmate's dive, play_grenade.rs) |
| Z | crouch / stand. Standing still: kneel (the stance's `stand2crouch`, then `rp_crouch_idle`; `crouch2stand` to get up). Moving: the crouch walk (`Sc_w1/w2_cr_walk`, `cr_back_walk` backing off while aiming; the clips' own root motion and footstep events), straight in from a run. Jump, dodge or sprint stands up. `BF_TEST_CROUCH=<s>` crouches from that time with `BF_TEST_GOTO` (held: asked for again each frame); `BF_TEST_CROUCH_KEY=<s>[,...]` presses Z once at each time, as the key toggles. In Tex's berserk Z does nothing and the press is dropped ([skills.md](skills.md)) |
| M | cycle the ground surface (changes footstep / landing / slide sounds) |
| click, then mouse | turn the aim, which the camera's view is (the game's right stick; the pitch held to +-60 degrees; no zoom wheel: each character's boom is the game's). Esc releases the mouse, in cutscenes too: cutscenes.md. See [camera.md](camera.md) |
| 1-4 | take control of Brutus / Flint / Hawk / Tex (on the test map whoever holds their places, any type, or no one: the squad menu, T, #302, [testtools.md](testtools.md)): their name shows in the middle of the screen in the game's heading font (atlas e0afcd52), with three red chevrons (the game's chevron texture 158f87c1) lighting up on the radar toward their portrait and the HUD open sound; 0.5 s on control passes, the close sound playing, and the name, shown 0.4 s whole, swells to 4x and fades over its last 0.3 s across the cut (the game's label, #88: squad.md), and the character you left drops back into the squad AI with a follow line ("I'm right behind you", "Following at a distance"...: chatter set f415ab02, from the shared voice bank `ml-sounds/en/common-en.tgz`; their speech icon shows by their portrait while they talk). Not while dead: the death camera hands over by itself |
| R | reload early: the game's X tapped (let go before its hold time - holding X is the use, the demo's E - or with nothing to hold it for: the player controller 0x1332a0 calls the reload, FUN_00120d40, at 0x134c9d, which starts it only when the clip isn't full and there's a reserve). R reloads on the press, the game's X on the release. The game also reloads by itself on an empty clip, see **Ammo, reload and the crosshair**. The stance's reload clip (`Sc_w1_reload` / `Sc_w2_reload`; upper body on the move, whole body standing); the clip reads 0 (panel red) until the clip's magazine-in event |
| G | use the item in the item box (the game's LT). A thrown grenade (Frag, Energy, Gas, Light, Sonic): hold to charge (the orange meter right of the crosshair, full in 0.625 s, held 0.5 s after the charge ends then fading out by 1 s; the held gun's crosshair stays as it is), let go to throw: 15 degrees above the camera's view (9 above the crosshair), a Frag at 17.8 m/s for a tap to 25.6 m/s full, the meter's bottom 30% throwing as a tap (#161: a full Frag looking 9 degrees down lands ~25 m out); the count drops at once, and the scope is left. A placed one (Roller, Sentry; m09_b's Thermo, #82): set down at the feet at the press. A Medkit (either): used at the press, "No need to heal" at full health. Nothing selected: "Inventory is empty". A Tab or B press while G is held drops the charge, and G does nothing more until it's let go. See **Grenades** and **The item box and the item list** below. A zone starts with the grenades, medkits and items its squad's placements carry, pooled (see **Items: the squad's starting inventory**); the OrgSen and the Hat do nothing yet (#21), but Brutus is told "Cannot use" |
| E (hold) | use: a gate's wall panel, from in front of it within 2.5 m, looking at it: "Hold [X] to activate panel." shows (the game's glyph line, X's glyph: E is the demo's X; [hints.md](hints.md)) and a blue ring marks its button; held 0.5 s, the gate opens and stays open. See "Gates and their wall panels" |
| Tab | the item box (the game's B button): a tap (let go within 0.25 s) steps to the next item carried, in the store's order (every grenade type, the Sentry and the Roller, the Medkit, the 60-point Medkit, the OrgSen, the Hat), round to the first; held 0.25 s, the item list opens round the box and the game pauses until Tab is let go: W or D (or the wheel up) the next item, S or A (or the wheel down) the previous, a step at once and another every 0.25 s while held; nothing else reaches the character meanwhile (movement, look, fire, G, the 1-4 keys). The item scrolled to stays. With a Medkit selected, G heals ("No need to heal" at full health). See **The item box and the item list** |
| I | in a mission, the MISSION OBJECTIVES menu (the game's BACK; the pad's Back too): the mission's objectives, " (completed)" or " (failed)" after each as its flags say; the game's clock stops under it. I again, Enter or Space (the pad's A or B) close it; while it's up no key or mouse press reaches the game (the closing one included). See [campaign.md](campaign.md) |
| V | the special skill on / off (the game's White button): Tex's Berserker, Brutus's Vengar, Hawk's Stealth, Flint's Targeting. Stamina (the blue bar) pays for it; on needs 10% of it, at 0 it switches itself off. Tex's Berserker draws his other gun into his left hand and fires both; in it Q and R end it instead of switching or reloading. Brutus's Vengar heals him, cuts the damage he takes, charges enemies down when he sprints into them, and turns the view grey with every living character a heat figure. Flint's Targeting widens the crosshair's lock. Hawk's Stealth fades her out over a second: the enemies lose her, the crosshair can't lock her, a shot or an item costs stamina. See [skills.md](skills.md) |
| B | the game's Black button, its Medkit shortcut: held, the 80-point Medkit is selected in the box and used as G would use it (healed, or "No need to heal" at full health), and let go the item before it comes back; "No Medkits" without one (the 60-point Medkit doesn't count); a grenade's charge is dropped; with Tex's berserk or Flint's targeting on it switches the skill off instead. The tutorial's button gates read it too ("Press [Black] to use Medkit.", [hints.md](hints.md)). In a test session B is the test tools' (an NPC's dummy / fight, [testtools.md](testtools.md)). See **The item box and the item list** |
| Esc | also the game's START for the tutorial's button gates ("Press [START] for game options."): the demo has no pause menu |
| Enter / Space, Q | on a tutorial panel (BASIC CONTROLS, HEADS-UP DISPLAY ...): Enter or Space (the pad's A) accepts it, Q (the pad's Y) skips the tutorial's later panels; nothing else closes it, and no press reaches the game under it (the game's clock stops). See [hints.md](hints.md) |

## Test map (`cargo run --bin bf_play -- --test`)

A flat test floor for trying things out, only reachable with the `--test` flag. It opens
straight into play: no loading screen, intro or menu (`src/bin/play_testmap.rs`).

* **Tex alone** (#302, the product owner's choice): the squad starts with the player's character
  (Tex, or `BF_CHARACTER`'s) and its other three slots empty. The squad menu (T) puts any
  character type in a slot, a real squad member (the formation, 1-4, the portraits and health
  channels, the hand-over at a death), empties one, or changes the player's own type where they
  stand ([testtools.md](testtools.md), "The squad"). `BF_TEST_FULL_SQUAD=1` starts with the four
  as before, `BF_TEST_SQUAD=<type>,<type>...` with those; a hook that acts on the squad's members
  (`BF_TEST_SELECT`, `BF_TEST_KILL`, `BF_TEST_TARGET` ...) starts it with the four too, so the
  recipes written for them run as before. Any other map (the L menu's levels, `--test
  BF_MAP=<level>`, the main game) keeps its own squad.
* **m04_a's music** (#302): its bank plays from the start, mixed as the music menu (U) mixes any
  level's (test_casp1_amb01, the ambience bed ambience19-2ch under it); U still changes it.

* **Every hand weapon on a rack**, floating and turning in front of the start: the 27 weapon
  definitions with a clip whose model loads (one per model), from the first mission's data,
  the multiplayer archives (`mp_common`, `mp1`-`mp8`) and m02_a, which holds the three
  campaign-only ones (A10 Bioreactive, Confed LZR-50, Jax-iP); about 0.7 s. (m09_a adds the
  Light grenade and sdm_e34 the missile rack for the object tool: about 1 s in all.) Each is a
  lying gun with a full clip, as the game's weapon pickups ([pickups.md](pickups.md), "Weapon
  pickups", #17): in reach (2.5 m across, 1.7 m up or down) the prompt offers the one most in
  front, "Hold [X] to swap <name>." with the blue ring, or "<character> cannot pick up <name>."
  with the red one for a class the character's slots don't take (Tex and Brutus refuse the
  pistols, Hawk and Flint the cannons). X (E) held 0.3 s swaps it in with the character's
  drop-and-pickup clip: the gun in hand drops at the feet (a lying gun too: its rounds go into the
  reserve first, which starts full here, so it keeps them), the new one comes with its full clip
  into the slot the game's rules pick (Tex's in hand; Flint's and Hawk's by class; Brutus's
  cannons to slot 2), its name over its count. The rack puts a new one in its place at once, and
  walking past it takes no rounds from it (`KeepClip`: the test map's convenience).
  `BF_TEST_GOTO=0.8,-4.3,0.8,-6 BF_TEST_GO_AT=999 BF_TEST_SWAP=2.0` stands Tex at the first row's
  fifth gun and swaps it.
  * The gun dropped there never vanishes: the flat floor plays as a mission (it isn't a
    multiplayer map in the game's level list), where a swap's dropped gun gets no vanish timer
    ([pickups.md](pickups.md), "The vanish timers").
  * The swap changes the guns a character holds, not its loadout (`Game::character_weapons`,
    which the rack's old walk-in respawn rewrote): a character made anew has its original guns
    again, as when play resumes from the map editor's edit mode (the squad comes back fresh,
    [editor.md](editor.md)) or a level switch brings the squad back.
  * Loading every level (about 6 s) adds no others: the rest are the creatures' built-in
    weapons (no model), Feral Cutter (the same model as another) and the bioreactive death's
    vaporise blast h_1f4a5a0a, which carries the iKhan-GPL's name (a model under 0.1 m; the gun
    itself, h_e9be70f0, is on the rack: #55).
  * `BF_TESTMAP_LEVELS=<level>,<level>...` loads a different list.
  * `BF_TESTMAP_LOG=1` lists every weapon on the rack with its type, clip and model size, and
    every hand weapon left off it, with why.
* **One of every pickup type** with a model (one per model) in a grid behind the rack.
  * They're dropped in from 0.3 m, upright as modelled but turned at random and tipped up to
    0.2 rad, so each settles as it would. A crate lands on its base; the Garo fruit (modelled
    on its point) and the cards (on their edges) fall over onto their sides.
  * They're inventory-objects as on a map, so medkits and fruit work as in "Health pickups",
    and everything is loose (kicked, thrown by blasts, tumbling).
* **Every squad grenade type**, a full stack of each (its stack-limit, 10): Frag, Gas,
  Sonic, Energy, Light, Sentry, Roller (the game's store order, #281). m09_a is
  loaded for the Light (only in m09_a/b/c/x). Tab steps through them. See **Grenades**.
* **Every character type of the data** (128) for the NPC spawner (#180): its levels define
  55; the other 73 come from the 31 level archives that define them, without those levels
  (`Game::load_type_assets`, about 0.9 s more; `BF_TEST_TYPES` picks a smaller set). See
  [testtools.md](testtools.md) and [character-types.md](character-types.md).
* **Instant kill**, on at the start (`BF_TEST_INSTANT_KILL=0`: off), K toggles it (`BF_TEST_TOGGLE_KILL=<s>[,<s>...]` presses it at those times). The player's shots and grenades kill any
  squad member they hurt, in one hit. The player still takes normal damage.
* **X kills the controlled character** on the spot, as any hurt does: the death cry and
  ragdoll, then the death camera and the hand-over to the next squad member (or, with nobody
  left, the camera stays on the body; on the test map's floor the player respawns, below).
  `BF_TEST_SUICIDE=<s>` presses it at that time. Its hurt is K's (`enemies::lethal`), so a type
  with a stamina shield in the player's slot (a deathmatch Shrike, #302) dies too; so does
  `BF_TEST_DIE`'s.
* **Dying alone respawns** (#302, the demo's rule for the test map, not the game's): with nobody
  to hand over to, 4 s after the death (the death camera's time to its hand-over) the player
  comes back where it stood, a fresh one of its type, its view and the squad's inventory kept.
  Only on the test map's floor, not on a level a test session loads ([testtools.md](testtools.md),
  "The squad").
* **The controls panel**, which the main game no longer shows: the keys (the tools' too) and
  the debug line (character, state, clip, surface, weapon), top left under the health bars. It
  starts hidden, with a small "H: help" at the bottom; H shows and hides it.
  `BF_TEST_HELP=<s>[;<s>...]` presses H then.
* The status message ("Instant kill off", the tools' messages) goes after its 2 s here too: it
  was counted down only on maps with doors, so on the flat floor it stayed.
* **Developer tools**: a free camera with a teleport (F), an NPC spawner (N), an object
  spawner (O, in the air from the free camera), any level's buildings and static scenery to put
  down, drawn and solid as in their levels (P, #185), the squad menu (T, #302), a sky picker (Y),
  a music picker (U) and a level switch (L), which takes the test session to any level and back. On a level the rack and the
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
character dies: death cry (chatter e1213623, `DEATH_CHATTER`: the game's CBT_DEAD handler FUN_00115f90 says it through FUN_00125f40 (0x125fb4) and the chatter gate (FUN_00140a60, 0x125fc9), or ed35ffae when the dying character's movement state +0x1f0 is 4, the state a landing leaves, FUN_0012b740: a death from a fall, inferred; the demo always says e1213623), the body goes limp as a ragdoll (verlet point
masses on the skeleton, pushed by the hit or blast; its limb bones are 0.09 m spheres that don't pass through each other, and every bone lies on the level's own floors, kept as far above them as the body is thick round it (`flesh`: of the skin each bone moves most, the distance from the bone's line that 80% of it is within, 0.04-0.2 m; a flat 5 cm let a thigh or the chest sink into the floor), and is kept out of its walls (the level's invisible blockers and hinters are neither for a corpse: only a character's body meets them, so a knocked-down one, which stands up where its ragdoll lies, keeps them, #298) - `BF_RAGDOLL_LOG=1` prints the closest limb pair and the lowest bone; the pose takes only the bones' turns from the simulation and keeps their rest offsets, so limbs don't stretch or twist; the ragdoll starts where the body is drawn, sole lift included (see "Standing on the floor"); the torso moves as two solid pieces (hips; chest with shoulders), stiffly joined at the waist, with the head held to the chest; knees and elbows (found by their Bip01 names) bend but don't fold past about 120 degrees or straighten past straight, and only one way: each joint keeps to its side of the line from the upper bone to the end, in the torso's own frame (hips for knees, chest for elbows), the side it was bent at death; a nearly straight limb uses knees forward, elbows back (forward from the toes); bodies keep 98.5% of their speed per substep, lose 60% of their sliding on the ground, and rest once still - but not on their side: a body come to rest propped on a shoulder and a hip (its chest's left-right axis more than 0.6 upright) is rolled on, its upper shoulder pushed 2 m/s toward its back (or front, if it leans that way), up to 3 times (the demo's choice). Shots hitting a body on the ground (dead or knocked down) push it: the first bone the shot passes within its thickness of is brought up to 3 m/s along the shot, and bones within 0.6 m less, so it jolts and rolls (a burst doesn't add up past that speed, so it doesn't drive the body across the ground); it wakes the body to settle again and it bleeds where it's hit, as the living do (blood effects, hit sound, a splat on the ground) (`BF_RAGDOLL_LOG` prints each body hit and the pelvis's position); an enemy of a type of its own (#129) has the game's ragdoll instead: bodies for its hit parts only, the rest rigid as posed at death, and the game's knee, elbow and hip limits ([enemies.md](enemies.md), "Death"); `BF_TEST_DIE=<s>` drops the player dead at that time, once; `BF_TEST_DIE=1` at 1 s). Both their guns drop, the one in hand and the one on their back (the ragdoll can't feel a stowed gun, so it propped bodies on their side): each falls from where it was, thrown by half of what killed them plus a 1.5 m/s hop, and tumbles as a loose object (see "Pickups are loose"), then lies as a weapon pickup with the rounds it had (pickups.md "Weapon pickups", #17). Then the portrait greys out and the blip leaves
the radar; if you die, the death camera circles your body and control passes to the next
living member after 4 s (see **Death camera** below; `BF_TEST_KILL=<character>` makes the
player shoot that member). The inventory is shared: the
grenades go with control. Every portrait's tab shows the follow-order arrows; a speech icon
appears beside the portrait while that member talks. The one you control is the player; the other
three are AI that keep a formation on your flanks and a little ahead (running / sprinting to
catch up, walking the last bit), look where you look, and
reload on their own. They never fire just because you do - only at enemies they can see (by the AI's sensor: their vision-range, x1.15 for one they know, 90 degrees either way of their facing, a clear line; [brain.md](brain.md) "Sight and contacts", #197), within their gun's range (the level's characters on another side, and the test map's NPCs set to fight, see [testtools.md](testtools.md)). Collision: everyone is a 0.4 m circle, pushed out of the pillars and apart from each other. Their footsteps and shots are heard quieter with distance, and they show as
green blips on the radar (turning with the camera, at the radar's scale: see **Radar** below). `BF_NO_SQUAD=1` (or `BF_DEATHMATCH=1`) plays deathmatch: alone, without the radar.

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
Without the tutorial's picture, the frame below is used. A HEALTH or STAMINA POWER walked over
when it can't do anything flashes the bar it would fill amber, four times in 1.2 s (the bar
widget's flash, 0x146900: `BarFlash`, [pickups.md](pickups.md) "Power-ups", #15).

HUD textures found for the panels: the health / energy frame is built from its corner
`h_0cdf5876` (32 × 16 end cap, mirrored four ways; it holds the step from the raised ends down to
the middle, whose level its inner column carries along), drawn as glass at 55% over a faint
tint. Its two channels are open in texel rows 6–11 and (mirrored) 20–25, columns 5–174; each bar
fills its channel exactly, and its icon is level with it. The bar fills (`h_e801997b` red,
`h_e11c7d04` blue, 4 × 8) are coloured in 6 of their 8 rows, matching the channels' 6; only those
rows are drawn. (Placed by hand, the bars sat over their channels' lines and left gaps at both
ends; drawn whole, the clear rows put the colour off-centre.) That corner isn't referenced in
the data and the demo picked it by shape; the HUD code names it, for split screen: h_0cdf5876
is the health bar's split-screen frame's left piece (0xb79f0 at 0xb7a53, with h_e0b02233 and
h_06a1d26e; full screen h_e5634eaf, h_eee38815, h_17470408). The item box's panel is the game's
own, the 64 × 64 h_e16927fb (FUN_000ab580, 0xab614; h_10891a71, 64 × 32, its split-screen one,
which the demo drew 9-sliced before #196): see **The item box and the item list**. (The
tutorial's `h_17f34cbe` is only a baked picture of the whole bar.) Radar health channels are filled as bands (distance from the centre and extent
along the diagonal, from the dark texels found), so the channels' highlight lines no longer
leave gaps.

HUD (`src/bin/play_hud.rs`), as in the game: health / energy top left, weapon panel top right
(the weapon's own icon from its definition, clip / the squad's reserve, or "clip Regen" for a
recharging gun; every weapon listed with names for 2.5 s after a switch, the held one in orange; in Tex's berserk two rows, the gun in hand over the second hand's, both orange, [skills.md](skills.md)), radar with the squad's portraits round it (Tex top, Hawk left, Flint right, Brutus bottom; only the members in the world: **Radar**, "The portraits"),
the item box bottom right (`play_itembox.rs`, **The item box and the item list**), the message line under the middle (the HUD entry's message widget: "No need to heal", "Inventory is empty"; the body font at 12 in (192, 255, 255) with a shadow, its last 0.5 s fading, FUN_000b32d0 / 0xb33b0), and the held weapon's own crosshair (its definition's `reticule-prefix` texture, e.g. f4ea7a92 for the MK-ASLT), 64 units square, above centre (y 192 of 480) - aiming and shots go
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
Standing with the stick at rest, the controlled character keeps its heading (#294): only the
stick's move and turn request and the aim turn it (FUN_001195c0 makes no request from a stick in
its dead zone). The AI's characters settle to a heading of their own (`face_yaw`: a squadmate
the leader's way, an enemy where its motion left it); a script's walk, a warp and a hit
reaction's end leave one on the controlled character too, and it used to turn back to it at
3.15 rad/s whenever it stood still (e34's opening walk; the tutorial's warps of Tex,
trigger_warp_tex_patrol and trigger_warp_tex_to_vsquad). `BF_HEADING_LOG=1` prints the
controlled character's heading every frame, what the step turned it, the aim's yaw, its place
and what may turn it standing (the stick, the aim, the gun raised, a slide, a throw), with a
`face_yaw` left on it. Standing still on slide-flagged ground built a slide up too, which
turned the character downhill (maps.md, "Sliding": `stands_still`). Checked (#294), each of
the four standing with no input, its heading unchanged to the fifth decimal:

* at sdm_e34's start and on its 36° face at (23, 10) (`BF_TEST_GOTO=23,10,23,10`), 95 s each
  in real time; before, Flint slid off the face at 41.6 s and Tex (another run) at 82.9 s,
  each turning about 180°;
* on that face at the capture's 15 fps (`BF_CAPTURE_FRAMES=900 BF_CAPTURE_EVERY=150`), 61.5 s:
  before, Flint slid at 1.3 s; the place moves 0.7-2.3 cm (the idle's root sway);
* after the tutorial's trigger_warp_tex_patrol (`BF_MAP=tutorial BF_TEST_GOTO=31,92,36,96
  BF_TEST_GO_AT=4 "BF_TRIGGER_FIRE=h_095ae7a3,11,2"`): Tex runs to the goal and keeps the
  heading he stops with (-1.99 rad, 5 s on), where before he turned back 140° to the warp's
  (+1.55) over 0.85 s;
* a run onto that face (#285's test, `BF_TEST_GOTO=30,10,-30,10`) still slides at 2.47 s.

Jumping uses the game's named clips (`Sc_w1_jump_crouch/launch/fall/land`, `Sc_4leg_w1_jump_*`;
names recovered from the motion scripts) with simple gravity; a jump pressed during a dodge is
buffered for 0.4 s.

Weapons (`src/bf/weapon.rs`): every character carries their real starting inventory
(objecttypes: Brutus Feral Cutter + Bower 20, Flint L-Shot-50 + Confed LZR-10, Hawk Foley 356
Tact + Confed LZR-23, Tex Confed LZR-23 + RVG50 Minigun; on a campaign zone each member's
placement's two instead, slot 1 in hand: see **Items: the squad's starting inventory**), with names from the English string
table, models from the level's weapon archetypes (24-byte static vertices; the minigun is a
compound whose barrel spins), and the definition's fire rate, fire sounds and range. A held gun
hangs from the trigger hand: the character's hand hardpoint lined up with the weapon's trigger
grip hardpoint (both point + w-x-y-z quaternion). That pairing makes every squad weapon point
straight ahead and level in the game's ready-pose clips, and puts the support hand on the
foregrip in the carry clips. Locomotion follows the weapon's stance. Each character has two motion
sets, `Sc_w1_*` and `Sc_w2_*` (idle, walk, run, dash, backpedal, reload, throw), and the game
picks one every frame from the gun in hand (char+0x5b0, FUN_0011aa20) by the character type's
pickup mode (`<limit-weapon h_fb08d4c6>`, the one that also chooses a picked-up gun's slot):

| Character | pickup mode | the set |
|---|---|---|
| Flint, Hawk | 1 | the slot's: slot 1 `Sc_w1_*`, slot 2 `Sc_w2_*` |
| Brutus, Tex | 2, 3 | the gun's class (its `weapon-type`, WTYPE 0 pistol, 1 rifle, 2 cannon): `Sc_w2_*` for a cannon, `Sc_w1_*` for any other |
| any other type | 0, 4 | `Sc_w1_*` |

So Brutus holds his rifle-class Bower 20 in his rifle stance, like the Feral Cutter, though it's
his second weapon, and a rifle Tex picks up into his cannon's slot is held as a rifle (#297: the
demo had keyed Tex's by the slot, from his `limit-weapon` slots, both "2 1"). Arms then carry each gun
the way the game animates it (Tex's minigun low in both hands, Flint's LZR-10 one-handed), and
aiming or firing switches to that set's `rp_*` ready-pose clips. The barrel is then turned onto the crosshair (yaw and pitch, two
passes, over the spine bones that carry the arm), so shots leave along the barrel (within 1
degree). The other weapon is stowed: the weapon's holster hardpoint (`h_10ab40ac`, on every gun) on
one of the character's three stow hardpoints (`weapon::STOW`), the one its slot's holder is on
(its place, `StowPlaces`, #297: the holders' +0x40 through their hardpoint getter 0x14ba70). Slot 1
starts on 0xff4c691b and slot 2 on 0xf4ccf78c, which hang Brutus's first gun from his pelvis
and his second diagonally across his back (on Bip01 Spine2), Flint's and Hawk's rifles across
their backs and their sidearms barrel-down on the hip (Bip01 R Thigh); only Tex's places move. His draw swaps them (FUN_00123fa0) and his switch clips' events move
them, so his stowed gun hangs vertical on his left ramp (0xff4c691b) whichever slot it's from, as
the takes have it (#61 take04: the LZR-50 of his slot 2; #68 take02: the LZR-23, then the
Minigun after Y), where the demo had hung slot 2's on his right ramp. Switching plays the character's own weapon-change overlay from its motion script (Flint and
Hawk `Sc_w1_2_w2` / `Sc_w2_2_w1`, Brutus `Sc_w1_2_w2rifle` / `Sc_w2rifle_2_w1`, Tex
`Sc_wc_rifle2cannon` / `Sc_wc_cannon2rifle`, whose ramps swing the gun over his shoulder) on the
upper body over whatever the legs are doing. The clips carry timed events (an event channel of
f32 time + name hash per event): at `drop_weapon` the held gun goes to its stow point, at the
grab event the other one comes to the hand, each with the weapon's handling (`pickup-sound`)
sound; no firing or aiming until the clip ends. Tex's clips swing his two ramps across his back
(the left one up to his right shoulder with the gun to draw, 0.50-0.83 s; the right one out and
over to his left side, 0.75-1.08 s; both back to rest between 1.25 and 1.33 s) and carry two more
events: the gun let go goes on his right ramp at `drop_weapon` and rides it over, ed585109
(1.20 s) hands it to Spine2's 0xe3307376, which sits where the left ramp rests, for the ramps'
snap back, and f18780b8 (1.37-1.40 s) onto the left ramp (FUN_0014d500, FUN_0014d570). Shots: muzzle flash and light,
recoil, tracers that fly to what the crosshair is on (ground or pillars) and a spark there.

`BF_WEAPON_LOG=1` (pickups.md) prints, whenever it changes, where each gun hangs (in the hand, the
second hand, or stowed at which hardpoint and bone) with its class and stance. Checked (#297) on
the test map (`--test`), with it:

* #61 take04's recreation (pickups.md, with `BF_TEST_SWITCH=10.0,13.0`): Tex's LZR-50 (slot 2)
  hangs on his left ramp before the swap and after it, as at take04's 1.0 s and 9.0 s (it had hung
  on his right one); after each Q the stowed gun is there again (the L-Shot-50 at 11.47 s, the
  LZR-50 at 14.47 s), having gone right ramp, Spine2, left ramp at the clip's drop, 1.20 and
  1.40 s;
* #68 take02's switch (`BF_TEST_WEAPON="Confed LZR-23;RVG50 Minigun" BF_START_WEAPON=1
  BF_TEST_GOTO=0,0,0,-1.5 BF_TEST_SWITCH=2.0,5.0`, every frame): the Minigun put on the right
  ramp at the drop (2.53 s) and carried over to the left as the take's (from 22.15 s on the right,
  22.55 s on the left; the demo's clip runs at its data's rate, slower than the take's: see
  **The crosshair** below);
* Tex swapping his Minigun for an MK-ASLT (`BF_TEST_GUN="MK-ASLT,3.6,0.3"`, the same weapons): the
  rifle goes to slot 2 (`Sc_w_drop_cannon_pickup_rifle`) and is held in `Sc_w1_*` (it had been
  `Sc_w2_*`: lower in the hands, as the Minigun; side view with `BF_VIEW_YAW=1.4
  BF_TEST_TRIGGER=4.0-5.0`, the carry and the ready pose against the old build's);
* Brutus (#52's recreation: the LZR-50 to slot 2 after the switch), Flint (an LZR-10 to slot 2)
  and Hawk (a Saryl-23 to slot 1), each with two Qs after: their guns' hardpoints and stances as
  before (each slot on its own, Flint's and Hawk's by the slot, Brutus's by the class).

### Radar (#228)

`play_hud.rs` (`update_squad_hud`, `update_radar_marks`), from the radar widget's code
(FUN_000ae580 and the functions below; [code-map.md](code-map.md), "HUD"):

* **Its scale** (FUN_000b0550): a world offset from the viewer, turned by the view, x 39 units
  (the max-range circle, the radar's +0x88 on a full screen) over the game options' `<radar>`
  max-range (120 m): 0.325 units a metre, each point cut to whole units. take07 agrees: the
  waypoint 17.4 m off drawn 5.4 units from the middle (5.65 by the code), the next, 55.5 m off,
  17.0 (18.0). The demo's 40 m across 44 units before #228 had no source.
* **The middle**: the viewer's dot, 180 180 180 (FUN_000ae7c0's colour for the HUD's own
  character, 0xae7f0: pale grey in take03 and take07; it was green). The code has no mode test:
  the squad deathmatch recordings' green middle is the squadmates' dots drawn over it, a few metres
  off at 0.325 units a metre.
* **The squad**: its living members in the world as green blips (FUN_000ae7c0's friend, 90 200 40;
  take11's marines; they were yellow) at that scale, one beyond the range on the circle (the game
  draws a rim marker for it: FUN_000ae7c0 through FUN_000b0ad0, not done).
* **The portraits** (#303): a squad slot shows its face, its follow-order tab and its health
  channel only while a member is in the world in it (`slot_member`; a dead one keeps its skull: dead
  members are still in the squad). A slot whose member the zone doesn't have shows nothing: the
  level spawns only the zone's members (its `h_ed54d255` names the others, [campaign.md](campaign.md)),
  so the tutorial and m01_a (h_ed54d255 "brutus hawk flint") show Tex alone, as take04 does
  (`todo/28-tutorial`, 60 s: Tex's face, tab and channel, the other three places dark), and m01_b
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
  level gives them or a radar-trigger sets ([triggers.md](triggers.md), "Radar-triggers"). Each
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

Verified (the tutorial, `BF_TEST_GOTO=10.64,44.15,22.28,34.31` into the first patrol's circle,
[triggers.md](triggers.md) "Test hooks"; crops of the radar against take07 and take03): the first
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

Not done: the radar's sound for a contact set with h_12b33046 (no tutorial trigger has it); the
radar's own placement and size on screen (take07's sits ~2 units right and ~4 lower) and the
dots' size; the squad's rim markers (above) and their white when selected (FUN_000c86d0); the
reach's easing by char+0x57c (above); contact 2's cursor on the screen (FUN_00052db0); which
list of characters the HUD's squad list is kept from (FUN_000bd790, "The portraits" above). A
test map dummy is no AI target, so nobody knows of a hostile dummy and it never shows.

### Ammo, reload and the crosshair (#114)

`src/bin/play_ammo.rs`, the HUD parts in `play_hud.rs`. Logic from `default.xbe` (functions cited
in the code), checked against the weapon takes in `todo/49-*` … `todo/68-*`.

* **The reserve is the squad's**, one count per ammo-type, shared by every member and every gun
  of that type (FUN_0022da80 looks the weapon's `ammo-type` up in the holder's inventory). The
  Foley 356 and the L-Shot share 11mm Ammo (type 1); the MK-ASLT and the Minigun High ROF Ammo
  (type 2). The panel reads "clip / reserve" (orange), e.g. "24 / 50" for Hawk's Foley.
  * Where a level starts the squad from its own placements (a spawn-trigger's template, or the
    squad placed in the world: [triggers.md](triggers.md), #220; every campaign zone), the
    reserve is the ammo they carry (`<inventory><h_10dd6db2 name count>`) and **nothing of any
    other type** (#281): the squad's store at the start (see **Items: the squad's starting
    inventory** below), whose IG_AMMO stacks are the ammo-types. The tutorial's Tex 500 High ROF
    (take03's "48 / 500"); e34's 11mm 40, Sonic, Particle and Rail 200, every other type 0, as
    xemu read the game's store at a fresh start (#196 notes); e40's 11mm 40, High ROF 600, Sonic
    200, Shotgun 80, Cutter 120 (39 and 199 once the opening's two reloads have taken their
    round: see the code 100 in [cutscenes.md](cutscenes.md)). A gun of a type nobody carries has
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
then, once; `BF_TEST_CROUCH_KEY=<s>[,...]` presses Z once at each time. These run after the
test tools' input gate, so they work under `BF_TEST_FREECAM` too (the gate drops the keyboard's
and `BF_TEST_FIRE`'s input, which `read_input` takes; #227 made that order explicit after a
new system flipped it);
`BF_AMMO_LOG=1` prints the reserve, each reload, dry shot (and the second hand's), recharged
round and the hint.

### Items: the squad's starting inventory (#281)

`src/bin/play_inventory.rs`. Logic from `default.xbe` (the functions are cited in the code and
have rows in [code-map.md](code-map.md)), the counts from the levels' squad placements, checked
against xemu's reads of the game's store at fresh starts (`todo/196-inventory-hud/notes.md`,
"Starting inventories") and the analyst's audit (`todo/inventory-audit/answer.md`, whose
Appendix A lists every campaign zone).

* **Each member's placement** (`levels-<zone>.xml`'s squad `<character-object>`, `<inventory>`)
  carries its two `<weapon>`s and `<h_10dd6db2 name count>` items: ammo, grenades, medkits, the
  OrgSen, the Hat, "Alien Technology Acquired!". The character's property handler applies them
  (FUN_0014dd50): the weapons into the slots, each item into the character's own store by
  FUN_0014bfe0, which keeps one stack per (category, index) (the item type's group-type and
  h_e5f51266) and holds it to the type's stack-limit.
* **One store for the squad.** Joining the squad (FUN_0005b9a0) gives a member the squad's
  store, its own merged into it first with the same clamp (FUN_0014c060). So the squad's items are
  every member's summed and held to a stack (m05_a's Frag 15 is 10, m09_b's Medkit 28 is 25), and
  every member's item box shows the same counts. The demo builds that store as the map starts
  (`StartStore`, `BF_INVENTORY_LOG=1` prints it) and hands it out: the IG_AMMO stacks are the
  reserve (above), the grenades and explosives `Player::grenades` (by the kit of the same label:
  m14_c's and m14_d's Sentry is h_f73de83d, which the kits load when a level defines no
  h_e5f1f063), the Medkit (5, 0) `Player::medkits`, the rest `Player::others`. They go with
  control, as the grenades always have.
* **What it gives** (`BF_INVENTORY_LOG=1` on all 39 campaign zones, against Appendix A: the
  same everywhere but where the appendix's sum passes a stack-limit, held to it: m05_a's Frag,
  m09_b's and m09_c's Medkit; m14_c's and m14_d's Sentry is the h_f73de83d kit):

  | Zone | Items | Reserve |
  |---|---|---|
  | tutorial | Frag 3, Gas 2 | High ROF 500 |
  | m01_a | Frag 5, Medkit 3 | High ROF 400, Shotgun 40 |
  | e34 | Sonic 4, Energy 5, Roller 5, Medkit 6, OrgSen 1 | 11mm 40, Sonic 200, Particle 200, Rail 200 |
  | e40 | Frag 5, Sonic 5, Energy 5, Roller 5, Medkit 20, OrgSen 1 | 11mm 40, High ROF 600, Sonic 200, Shotgun 80, Cutter 120 |
  | m09_a | Frag 10, Sonic 6, Energy 8, Light 4, Roller 6, Medkit 10, OrgSen 1 | 11mm 40, Sonic 200 (480 carried), Particle 200, Rail 200, Bio 200 (380) |

  e34's and e40's are what xemu read in the game's store (e40's ammo after its opening: 11mm 39,
  Sonic 199). A map without squad placements keeps the demo's own stock: the test map a full
  stack of every grenade type, the deathmatch maps 3 Frags (the sdm loadout is #144's).
* **The OrgSen and the Hat** are items of the box (their type's h_1852ab6c lists them; "Alien
  Technology Acquired!" is carried but not listed), with their own icons and names, the count
  shown from 2 as for every item (the game's box: #196 take26). What they do is #21's (the
  OrgSen) and the Hat's own: G with one selected does nothing yet. A character whose skill
  refuses its use type can't use it (the skill's +0x1c: Brutus's FUN_0013c160 refuses
  IOU_ATTACH_TO_FACE, the OrgSen, and IOU_ATTACH_TO_EQUIP_SLOT, the Hat): his box shows it red
  with the icon grey, and G says "Cannot use" (h_e784869e), as #196 take26 shows.
* **The gun in hand** at a start is slot 1: a placement leaves none in hand, and the character's
  activation (vtable +0xa4) draws the first slot with a gun (FUN_00123fa0). A zone's opening then
  switches some by its script's code 100 (CT_TOGGLE_WEAPON, [cutscenes.md](cutscenes.md)): after
  e34's opening Tex holds the Confed LZR-50 and Flint the Jax-iP, after e40's Tex the Jax-iC,
  Hawk the Jax-iR and Brutus the PU Cutter, as xemu read them and take27 / take30 show ("99 Regen",
  "36 / 199"). With `BF_TEST_GOTO` or `BF_SCRIPT` the opening doesn't play, so slot 1 stays in
  hand.
* **The first item shown** is the first listable stack in the game's store order (category,
  index: Frag 2/0, Gas 2/1, Sonic 2/4, Energy 2/5, Light 2/32, Sentry 3/1, Roller 3/4, Medkit
  5/0, the 60-point Medkit 5/2, OrgSen 6/0, Hat 6/3; `load_kits` sorts the grenade kits by it):
  the front's selection starts at none and FUN_0014db60 steps it to the first (#196). Frag on the
  tutorial, m01_a, e40 and m09_a, Sonic on e34 (take27's "Sonic 4"), Gas on e16 and e23. Its NEW
  flag is clear and every other stack's set (**The item box and the item list**). Tab steps in
  that order.
* **Not done**: carrying the store from zone to zone (the save game's `item-%d` pool and each
  member's weapons, clips and hit points: FUN_000816f0, FUN_00081d10, FUN_00129080,
  FUN_00089ea0; #231), so a zone the game only reaches from the one before starts with what its
  own placements carry (m03_b, m03_c, m08_b: no item, and the reserve's full stacks, the stopgap
  above); the order of the joins and the mode's
  clear on a zone whose squad is placed in the world (m07_a, m09_a: the demo takes the
  placements', a live read would settle it); the OrgSen's and the Hat's uses; a Hat picked up (only medkits, fruit and DNA canisters are taken).
* **The switch sounds in a cutscene** (#282): the controlled character's scripted toggle makes
  them, as the demo plays them (e40's Tex at 7.7 s, e34's at 17.98 s). From the code: the squad's
  HUD entries, with their weapon panels, update every frame of play unless a modal menu is open
  (the session's frame update 0x7a140), every member's entry included; the sound's gate is
  FUN_000bb220's, a member whom a player slot controls (0x8cf60), not "the entry's character".
  The cutscene's HUD hide is the HUD's alpha (1 - 2 x the letterbox amount, FUN_000b8e30), which
  no widget's hidden bit follows. The takes can't confirm it: #196's take30 starts 10.2 s into
  e40's opening, after the toggles, and take27 (e34, Tex's toggle at 8.6 s of the take) has no
  match for either sound, but a pair at the level #73's take02 has them is lost under the
  opening's music and voices there too (a masking test, injected and correlated).

Test hooks: `BF_INVENTORY_LOG=1` prints each member's placement items, the pooled store, what the
controlled character is given and each use of an item that isn't a grenade or a Medkit;
`BF_TEST_ITEM=<label>` selects the carried item of that name at the start (`OrgSen`, `Medkit`,
`Medkit60`, `Sonic`); `BF_TEST_ITEM_USE=<s>[,<s>...]` presses G for a frame at each time for an
item that isn't a grenade; `BF_TEST_INV` and `BF_TEST_INV_NEW` set the pool and its NEW flags at
the start (**The item box and the item list**).

Verified (`BF_INVENTORY_LOG=1 BF_AMMO_LOG=1`, 15 fps captures in the scratchpad, against the
takes in `todo/196-inventory-hud/`): the five zones above print the table's counts; e34 played
through its opening (`BF_MAP=e34`, 375 frames): the script's 100s switch Flint at 13.0 s and Tex
at 17.9 s, and the first controllable frame reads "99 Regen" as take27's; e40 through its
opening: Hawk, Tex and Brutus switch at 7.1-8.0 s, Flint's and Tex's reloads take 11mm 40 -> 39
and Sonic 200 -> 199, and the HUD reads "36 / 199" and "Frag 5" as take30 (14 s); with
`BF_TEST_SELECT=2,24.5` Hawk's reads "24 / 199" with the Jax-iR held low at her right and no gun
on her back, as `stills/still-e40-hawk.png`; on e34 (`BF_TEST_SELECT=1,23.5`) Flint's panel shows
the Jax-iP. m09_a (`BF_SCRIPT=off "BF_TRIGGER_FIRE=trigger_letterbox92,12,0.05"`): Tex's A50
"40 / 200" and "Frag 10"; m01_a: "48 / 400" and "Frag 5". The OrgSen box on Tex
(`BF_TEST_ITEM=OrgSen`, e34's range spot) tiled beside take26's at 1.5 s: the same icon and name
in the same place; Brutus's (`BF_CHARACTER=0 BF_TEST_ITEM_USE=1`) is red and grey with "Cannot
use" over him, as take26 at 7 s. Regressions: the tutorial's "48 / 500" and "Frag 3"; e40's Frag
thrown, 5 -> 4; e34's Medkit used at 50 HP, 6 -> 5 and healed; the test map's full stacks and its
weapon switch (`BF_TEST_SWITCH=1.5`, the switch now `start_switch`); sdm_e34's 3 Frags and full
stacks. After the kits' reorder (the review round): e34's box "Sonic 4" at the range spot, tiled
beside take27's at 14 s; e16's and e23's first item Gas; the test map's kits Frag, Gas, Sonic,
Energy, Light, Sentry, Roller; m14_c's Sentry (3, 3) before its Roller; m03_b, m03_c and m08_b
with the full stacks (the stopgap), m02_b with its 11mm 40 and Cutter 75 alone. e40's opening with
`BF_SOUND_LOG=1`: Hawk's and Brutus's toggles silent, Tex's ff820fda and, 0.65 s on, e1e97460,
beside the opening's own `trigger_sound_weapon_select` (e1e97460 at 7.7 s).

### The item box and the item list (#196)

`src/bin/play_itembox.rs`, from the player controller's item section (0x1332a0), the HUD item
widget (FUN_000ab8d0, its update 0xabc60 / FUN_000ac220) and the inventory front's selection
(FUN_0014d960, FUN_0014db60); each function is cited in the code and has a row in
[code-map.md](code-map.md). Measured against the takes in `todo/196-inventory-hud/` (its
`spec.md`, AC-01 to AC-35).

* **The selection** is the inventory front's wanted pair (`Player::item`; none is the game's
  (10, -1)), one stack of the squad's store at a time. The order is the store's, category then
  index, round: Frag, Gas, Sonic, Energy, Light, Sentry, Roller, Medkit (80), Medkit (60),
  OrgSen, Hat. A stack is listable while it holds any (one that reaches 0 loses its type) and its
  type's h_1852ab6c lists it. It goes with control: a squad switch or the death camera's hand-over
  copies it to the new member (FUN_000c8fa0; take17, take29: every member's box reads the item).
* **Each frame of the world** (FUN_0014db60): a none selection becomes the first listable stack
  (a level's start: e34's Sonic 4, e40's Frag 5, the tutorial's Frag 3); the shown pair, which
  "can use now" checks, follows the wanted one unless a throw or a Medkit's use is under way; a
  shown stack that ran out sets both to none, so the box is blank for that frame and shows the
  first stack the next (take09: "Sonic", one blank frame, "Frag 5"). A pickup never moves the
  selection (take34: the box stays on the Sonic while the Medkit stack fills).
* **Tab, the B button** (0x1340e1): held, its timer counts and a grenade's charge stays 0; let go
  under 0.25 s (0x3a5028) it steps to the next item, with the step's sound when it changed
  (FUN_000abfc0); at 0.25 s the item list opens (FUN_000ac080: its sound, the game paused,
  `PAUSE_ITEMS`) and stays until Tab is let go (FUN_000ac160: its sound, unpaused). Nothing is
  picked at the release: the item is what the list was scrolled to. The pause stops the world as
  the menus do (`frame_dt` 0: the characters, grenades in the air, effects, timers); the item
  widget runs on the HUD's own clock meanwhile (`hud_dt`: take02 slides it while the app clock is
  still). Movement, look, fire, G, B, the skill key and the 1-4 keys reach nothing while it's
  open.
* **The scroll** (FUN_000ac370), while the list is out past 0.75 and Tab is held: the stick past
  0.5 on its larger axis, right or up the next item, left or down the previous; a step at once,
  then one each time its ramp passes 1 at 4 a second (0.25 s and a frame: 8 frames at 30 a
  second, take08). The keyboard's stick: W or D the next, S or A the previous (the product
  owner's call D1); the wheel a step a notch, up the next (D2): the demo's, as the game has no
  wheel, so its notches are queued while the list is open and stepped one a frame once the
  scroll runs (none lost to the slide's first 0.375 s or to two in a frame; a fine wheel's
  fractions of a line and a touchpad's pixels, 100 a notch, add up to whole ones). The pad's
  left stick and d-pad are #154's (`ItemInput`, which the keyboard fills now).
* **G, the game's LT**: nothing selected, "Inventory is empty" (shown again each frame it's held,
  0x1349c3, so 3 s from the release); the shown pair not the wanted one (a Tab tap during a
  throw), "Not ready" (code only); else a grenade charges and is thrown on the release, the rest
  are used on the press. A Tab or B press while G is held leaves it dead until it's let go
  (ctrl+0x68: take19, take12), and so does a use (ctrl+0xa0 bit 7); once G is up the lock is
  free again (0x1346a7), so a use never leaves G or B dead.
* **B, the Black button** (0x134316): a charge dropped; Tex's or Flint's skill on: switched off
  instead (#35); no Medkit (5, 0): "No Medkits" (the 60-point one doesn't count, take11); else the
  Medkit selected (the item before it saved) and used the next frame as if G were pressed
  ("No need to heal" at full health), and on the release the saved item back (take10: the box
  Medkit, the Sonic back as Black is let go). Held on, another Medkit each time the last one's
  use is over and the character is still hurt (FUN_001210c0 with the Medkit's own "can use",
  0x147070; code only). The controller is the player's, not the character's: a hand-over during
  the hold gives the saved item back to the new member at the release (FUN_000c8fa0 leaves it).
* **The box** (FUN_000ab460, FUN_000ab580, FUN_000ab740): at (528, 375), 64 × 64 units, hidden
  while nothing's selected (take21). Its panel h_e16927fb (64 × 64, white); the type's icon
  h_e5ec3f1f 64 × 64 at its corner, grey while the character can't use it now; the name (the
  type's stringtable-name) centred, its top at 45; the count right-aligned 4 in from the edge,
  its top at 31, shown from 2 up; the body font at 12 (the 12-point record, play_fonts.rs) with a
  shadow; (192, 215, 255), or (150, 70, 70) while it can't be used now: a Medkit at full health,
  the OrgSen or the Hat for Brutus (take01, take10, take26). A dead member's box isn't red
  (take18). Its character dead, the box fades out from the death's frame (0xabc60 sets its fade
  rate, +0x11, to 127 a second; FUN_000ba730 takes it off the fade level, +0x12, in whole
  levels: 5 a frame at the game's 30, so out in 1.7 s) and stays out through the death camera;
  the next member's box is at full at once (take18: the icon gone 1.66-1.73 s of the game's
  clock after the death, the picture frozen twice just after it as the game hitched; Flint's
  box at full at 6.9 s).
* **The list** (FUN_000ac500, FUN_000ac5e0, FUN_000ac7f0): its open level rises 2 a second while
  it's open and falls 3 after, on the HUD's clock (out in 0.5 s, gone 0.33 s after the release);
  up the next item and the one after, left the previous and the one before, at int(open × 70)
  and twice that from the box, faded in as open³; their panels (30, 30, 30) at 153. A slot showing
  an earlier one's stack is hidden in the order box, left 1, up 1, left 2, up 2: one stack shows
  no slot, two left 1 only, three left 1 and up 1, four all but up 2 (take25's logs).
* **NEW** (0xad0c0): every stack has the flag from its making (FUN_0014ac90); the first slot to
  show it (the box, or a list slot as the list opens) clears it and runs its label, 2 s on the
  HUD's clock: "NEW" (h_0ba41874) in (255, 160, 106) at (10, 2), on while fract(2 × time left)
  ≥ 0.3 (0.35 s on, 0.15 s off), its alpha 255 × min(1, 2 × time left). A slot's label stops when
  its stack changes. At a level's start the stack shown first is clear and every other set
  (take27: the first box has no label; take28 / take32: the first taps show NEW). A hidden list
  slot counts too, in the widget's child order box, up 1, up 2, left 1, left 2 (FUN_000b21f0 and
  0xad0c0 don't test the slot's hidden flag): with Sonic selected and Energy and Roller NEW, the
  hidden up 2's Roller takes Roller's NEW and the left 1 that shows Roller has no label (code
  only: no take has a short list with NEW stacks).
* **The messages**: the item ones show for 3 s (FUN_000bcfe0(entry, 3.0)), restarted by each
  press, on the message line (above): "Inventory is empty", "No Medkits", "No need to heal",
  "Cannot use", "Not ready". take26's "Cannot use" stays on through presses 3 s apart (on 6.2
  to 15.15 s of video) and is gone 2.8 s of the game's clock after the last (the watch log's
  clk: 682.48 s at the press, 685.27 as the line's ink goes), where the demo's takes 2.87 s to
  the same threshold; the take's notes' "~6.5 s each" is that run of presses.
* **The sounds** (FUN_000bb220, 2D at their library volume, h_eff0deab 0.7): the list's open
  0x18fc2db8 and close 0xef4bd74d, the next item 0xf8f93c5e and the previous 0x1ab2fc23 (only when
  the item changed). Logged with `BF_SOUND_LOG`; not heard in the takes (their game sound was at
  the noise floor), so their loudness isn't checked.
* **Not done**: the pad (#154); the weapon panel and the skill's name showing while the list is
  open (take02, the product owner's call D8: a follow-up); split screen (64 × 32, no icon); the
  OrgSen's and the Hat's uses (#21); "Can't use this item." (FUN_001301e0's use types per
  character, not traced: only Brutus's refusals are in, his skill's "Cannot use"); "Not ready"
  for a character that isn't free (FUN_000abbb0 asks FUN_00117210 too: G pressed during a throw's
  motion, a Medkit's use, a reload or Flint's targeting; only the pair's part is in); the rest of
  the HUD's death fade (the health bars, the weapon panel); whether the HUD's other timers (the
  message line, the throw meter) run under the list's pause (they stop in the demo); the sound
  groups paused under it (FUN_0007a360); Hawk's special pair (10, -2); the two silent LT presses
  of take13 / take15 (the code's alternating ctrl+0x68 explains them, code-map's 0x1332a0 row;
  not copied, as the spec asks).

Test hooks (the hooks' clock is the sim time plus the time the list's pause held it, so a hold's
end comes while the world is stopped):
* `BF_TEST_ITEM_INPUT="<s>:<input>:<hold s>;..."` holds an input from that time: `tab`, `lt` (G),
  `black` (B), `next`, `prev` (the stick); a hold shorter than a frame still shows for one.
  `<s>:wheel:<notches>` turns the wheel that many notches in one frame (up +, down -).
* `BF_TEST_ITEM_LIST=1` holds Tab from the start (the list opens and the game pauses for good).
* `BF_TEST_NEXT_GRENADE=<s>[,<s>...]` taps Tab (0.1 s) at each time (the T key it pressed is
  gone: the product owner's call D3). `BF_TEST_THROW` (with `BF_TEST_GOTO`) holds G.
* `BF_TEST_INV=<name>:<n>,...` sets the pool at the start: a grenade type's label, `Medkit` (the
  (5, 0) stack), `Medkit60` (the (5, 2) one), another listed item's label (`OrgSen`, `Hat`); 0
  empties a stack, the rest keep the store's; a grenade type the level defines but neither the
  squad nor SQUAD_GRENADES has is loaded for it (e34's Sentry). `BF_TEST_INV_NEW=<name>,...`: the
  stacks still NEW at the start, every other clear. #196's nine-stack pool:
  `BF_TEST_INV=Frag:5,Gas:5,Sonic:4,Energy:5,Sentry:3,Roller:4,Medkit:2,Medkit60:2,OrgSen:1
  BF_TEST_INV_NEW=Gas,Energy,Sentry,Medkit60 BF_TEST_ITEM=Sonic`.
* `BF_ITEM_LOG=1` prints the controls' events (taps, the list, Black, uses, refusals, a stack
  running out, NEW) and the state on every change and every 0.1 s while the list is out: the hooks'
  time, the sim time and the HUD's, the open level, the timers, the selection and the shown pair
  as (category, index), the five slots (box, up 1, up 2, left 1, left 2; `h` hidden), the lock,
  Black's saved item, the counts with their NEW flags, the labels' times.

Verified (e34's range spot `BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-53.56,30.81 BF_TEST_SWEEP=167.8
BF_CAMERA_PITCH=-0.0093 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`, the nine-stack pool where
it says, `BF_ITEM_LOG=1`; frames tiled beside the takes'):
* The box: "Sonic 4" beside take25 at 1.5 s: the icon's orange at the same box-relative x
  17.3-45.3, y 9.0; the name's and the count's ink within a unit of the take's. Ten taps 1.5 s
  apart (take01): Energy NEW, Sentry NEW, Roller, Medkit, Medkit (60) NEW, OrgSen, Frag, Gas NEW,
  Sonic, Energy, each with the next sound; the Medkits grey and red at full health, the OrgSen
  teal, as take01's frames.
* The list, Tab held 2.0-5.0 at 30 fps: open 0.267 s after the press (8 frames), the sim time
  still from then to the release; the slots 2,4 2,5 3,3 2,1 2,0 as take25; NEW on Gas, Energy and
  Sentry as it opens (take24), their labels blinking and gone 2 s after the open, frame for frame
  as take02 at 10 fps; out at 0.533 s, back in 0.33 s. Two, three and four stacks: take25's slot
  logs. A Frag thrown before the open hangs in the air through it (`BF_GRENADE_LOG`: down 0.33 s
  of sim time after it left the hand, which spanned the 2.8 s pause).
* The scroll (take05 / take08): steps 0.267 s apart while held, two for a 0.4 s push, none in
  the slide's first 0.375 s, the item kept at the release.
* Taps of 0.10, 0.18, 0.22 s step, 0.30 and 0.40 s open the list (take04; at 30 fps a 0.24 s hold
  is 8 frames, 0.267 s, and opens it). A tap while a Frag charges: the charge dropped, the Gas
  selected, nothing thrown though G stays down (take19), a later press throws the Gas. A tap in
  the throw's motion: the box shows the Gas at once, the shown pair follows as the grenade leaves
  the hand, G meanwhile "Not ready".
* Black at 60 hp: Medkit selected and used (6 to 5, 115 hp), the Sonic back at the release; at
  full health the grey and red Medkit and "No need to heal" (take10, its message's ink at the
  same x 269.3-370.0); the (5, 0) stack empty: "No Medkits", the box unchanged (take11); during a
  charge with G held to 4.0: the charge dropped, the Medkit used, the Frag back, no throw (take12);
  Tex's berserk on: the skill switched off.
* The last Sonic thrown: "Sonic", one blank frame, "Frag 5" (take09); a third throw takes a Frag.
  Nothing carried: no box, "Inventory is empty" at y 292.0-306.7 as take21; Tab held opens the
  list with no slot. The tutorial's medkits taken: the box stays "Frag 3", taps reach Gas and the
  Medkit, each NEW (take34). Brutus with the OrgSen: grey, red, "Cannot use" (take26). Tex taps
  to Energy, control to Hawk: her box "Energy 5" (take17). NEW's blink at 60 fps: 21 frames on,
  9 off, three times, then the fade (take23: 22 / 11 / 24 / 8 / 24 / 9 video frames at the take's
  0.9 speed). e40's first item Frag 5, the tutorial's Frag 3.
* Regressions: `BF_TEST_MEDKIT`, the 60-point Medkit healing 60, the test map's kits and
  `BF_TEST_NEXT_GRENADE`, `BF_AUTOPILOT`'s throw at 29 s, `cargo test --bin bf_play`.
* The use lock (`BF_TEST_HEALTH=10`): the Medkit selected, G heals (90), Black after it heals
  (115), Black again says "No need to heal" (the log: the lock on the Medkit at the press,
  "another" while G stays down, free at its release); a one-frame G (`BF_TEST_ITEM_USE`) then
  Black; two Black presses with the Medkit current; Tab three times to the Medkit, G, then Black.
  Black held 3 s: a Medkit (90), another once the first use was over (115), the Sonic back at the
  release. A switch to Hawk asked during a Black hold (`BF_TEST_SELECT=2,1.2`): her box on the
  Medkit until Black is let go, then "Sonic 4".
* NEW in a hidden slot: Sonic, Energy and Roller (`BF_TEST_INV=Medkit:0,OrgSen:0`), Tab held: up
  1's Energy shows NEW, left 1's Roller none (the log: NEW on the hidden up 2).
* Brutus's "Cannot use" with G presses 3 s apart: on 3 s from each press, its ink gone 2.87 s
  after the last (take26: 2.8 s of the game's clock).
* The death box (`BF_TEST_DIE=2.0`, the Energy selected, 30 fps): the icon's and the name's
  brightness fall in a line, out 1.7 s after the death (take18: 1.66-1.73 s of the game's clock),
  out through the death camera, the next member's box at full at the hand-over.
* The wheel (`<s>:wheel:2` 0.1 s after the list opened): both notches stepped once the scroll ran,
  a frame apart; one back later.

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
  +-0.01% on each 30 Hz frame (`TICK_JITTER`), which moves no other period. Its random numbers
  are the frame clock's own (`FireState::clock_seed`, #54): drawn from the gun's stream between
  its shots' draws, a cycle of a fixed number of draws sampled the LCG at a fixed stride, and at
  the Foley's 16 (a shot's 3 accuracy draws and 2 effect seeds, then 11 frames) a cycle's frames
  ran short together every cycle: 11 frames every round, 2.73 a second against the takes'
  2.79-2.93; now 10-11, 2.89 a second. (Every gun's random patterns moved with it, not their
  spread.) In
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
  in the demo. From idle the gun is raised first, for the character's own ready time (#140;
  todo/140-footage-timing/answer.md §1): FUN_00123ea0 starts the ready timer at 4 + its motion
  set's `pc-transition-ready in-time` while a player controls it, else its
  `npc-transition-ready in-time` (FUN_0010fce0; `characters/ms_<name>.xml`, h_f599b114 /
  h_170db5cd), and the fire command fires nothing while the timer is over 4. The squad's pc
  in-time is 0.1 s, so a tap's round comes on the 3rd or 4th 30 Hz game frame after the press,
  0.10-0.13 s (`raise`: the in-time and one game frame, as the game tests the timer before it
  counts it down); as AI the squad's is 0.2 s (Brutus 0.3), e40's Shrike Soldier 0.3 and Heavy
  0.4. The demo's 0.2 s before was fitted to the takes' 0.19-0.24 s press-to-shot, which ran
  from the pad input to the 10 Hz watch log, so it held the rig's 0.02-0.06 s input delay and
  the log's 0-0.1 s lateness on top of the game's 0.10-0.13 (xemu.md, "Timing in the takes").
  The ready pose blends in over the same in-time and out over the out-time (FUN_0011a440: 0.2 s
  for the squad), where other clip changes take the demo's 0.2 s. An AI character raises its gun
  as it takes up a target (the attack check's FUN_00123ea0, 0x2509c; play.rs `ai_fire`), so its
  first round waits its npc in-time, which runs alongside the demo's own opening delay (0.2-0.8
  s, `ai_delay`) and is mostly hidden by it. After a shot the gun stays up 4 s
  (`READY_HOLD`: FUN_00123f60 keeps +0x7dc at 4 or more, medium confidence), so a tap within it
  fires in the frame the press is seen (the takes' taps 2.2 s apart logged 0.07-0.11 s after the
  input: the same two delays). Checked (`-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=0,4,0,4
  BF_TEST_WEAPON="Bower 20" BF_TEST_TRIGGER=2.0-2.08,4.0-4.08,6.0-6.08 BF_SHOT_LOG=1`): at
  `BF_CAPTURE_FPS=60` the press is seen at 2.03 ("raises the gun: the first round in 0.1 s
  (pc-transition-ready in-time)") and the round goes at 2.13, +0.10; the taps at 4 and 6 s fire
  in their own frame (4.03, 6.03). At 15 fps steps the first is seen at 2.07 and fires at 2.13
  (the next step); before #140 these were +0.20 and +0.03 at 60 fps. At 15 fps a
  `BF_TEST_TRIGGER` window starting on a frame's time (e.g. 3.0 s) can be seen only on the next
  step (3.067 s). An e40 Shrike Soldier on the test map (`BF_TEST_NPC=...,fight`) logs "raises
  the gun: the first round in 0.3 s (npc-transition-ready in-time)" as it takes Tex up, and fires
  0.87 s later (its opening delay).
* **The burst counter** is cleared by a reload starting (FUN_0022daf0), a dry click
  (FUN_002327f0) and a switch (FUN_0022dc00), as in the game.
* **The seed** of a shooter's fire loop is the demo's own hash, character x 7919 + NPC number x
  104729 (`play.rs` `shot_seed`), in wrapping arithmetic (#229): an NPC numbered above ~41,010
  overflowed a u32 and panicked a debug build. The seeds below that are unchanged, so the shot
  patterns are too. Checked by `cargo test --bin bf_play shot_seed`.
* **Bursts and pellets**. A trigger cycle is h_e4076713 shots, h_e6c60892 s apart. At 0 s apart
  (the Bower 20's 6) they all go in the same frame for one round and one report, and each after
  the first is turned from the one before (FUN_0022e1d0: the pellets walk away from the first)
  **in the world's frame** (#49): the aim d becomes Ry(a2) Rx(a1) d, a1 then a2 drawn uniform in
  +-h_e704fd69 degrees (6 for the Bower, 3.5 for the MNR-7's darts), about the world's x axis and
  then its vertical (FUN_0013e800 builds Rx, FUN_0001fa20 turns it by Ry, FUN_002334b0 multiplies
  the aim's frame by it, FUN_0001f9c0 takes the frame's -z: every instruction read for #49's spec,
  §4). So a heading along x keeps a shell's pellets nearly level (the x turn spins the aim about
  itself) and one along z scatters them up and down as much as across; across, every heading
  spreads the same. The demo turned the pitch about the gun's own right before: up and down as
  much as across at every heading.
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
    field of view, came out 1.5-2x tighter than this. The ~74 degrees vertical that the data's
    `<camera> fov="90"` would give, and under which the MK, Minigun and Foley matched within
    ~10%, isn't the game's view: the follow camera's is 70 across its 4:3 picture, 55.4 down
    (#126's live read, [camera.md](camera.md)). Read at 55.4 the clusters are still about 1.2-1.6x
    tighter than this (1.5-2 / 1.27): the accuracy fit is unexplained again (for the weapons'
    tickets).
  * The Bower's "~2x too wide horizontally" (read from the tank's holes, where the pellets past
    ~+-7 degrees miss the 2.6 m cylinder and leave none) is settled by the walk's frame (#49, above):
    its takes' sparks at the 25 m cliff (take09, heading -170) spread 10.3 degrees across (sd 3.8)
    and 0.8 up and down, the world-frame rule's 9.3 (sd 3.5) and 1.7 (0.7-2.9), where the old
    gun-frame turn gave 9.3 up and down.
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
  bullet's h_06a27365 decal (0.3 x 0.3 m holes and scorches, read as full sizes; the decal's
  life and fade, h_199870ec / h_fe4e1d82: 25 s fading over the last 5 for most, 60 s for the
  bio guns' green h_1ce6db03) laid on the
  surface it met: the map's collision triangle (`Arena::ray_normal`) or, on the flat test floor,
  the ground or a pillar's side. Up to 160 stay (a guess). A bolt's hole comes with its hit, on
  its sweep (**Bolts and their hits** below). An instant shot, as a bolt's sweep and a missile,
  passes through the level's invisible blockers and hinters to what's behind them (#298: the
  weapons' filter, PID mask 0xfffffff8, leaves them out; maps.md, "Blockers and hinters"), and a
  breakable object's own blocker no longer takes the hit meant for the object. A hole on a
  breakable object goes with it when it breaks (#301: scenery.md, "Breaking"), the hole of the
  shot that breaks it too: the shot finds the object with its hit (`Shot::breakable`), as the
  game's hit record holds it.
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
* **Muzzles** (#246; `bf/weapon.rs`, `WeaponDef::muzzles`). The muzzle block's h_e41cb542 is a
  list of hardpoint names (the weapon data's vector at +0x140), and the game uses every one: the
  muzzle effect is attached at each (FUN_00232060), each one's aim is turned every frame
  (FUN_0022e4c0), and a shot leaves from the muzzle the weapon's index (+0x1fc) points at, which
  then moves on to the next, back to the first past the last (FUN_0022f5e0;
  `WeaponDef::next_muzzle`). The muzzle effect that follows the shot is the one at the index as
  moved (FUN_0022f170 reads +0x1fc after it): the flash shows at the barrel that fires next.
  Every hand-held gun names one, MUZZLE (h_ecc0aa78), so its index never moves and the demo's
  guns fire from it as before (`WeaponDef::muzzle`); the four turrets (object-type 29: e05, e16,
  m03_b, m08_b, m09_b/c/x, m14_a-d) name two, h_f9eed613 and h_e2e787a9, of which the reader
  kept only the first; 14 definitions without a model name none. The demo has no turrets yet,
  so the turn is read from the code and not seen. `BF_DUMP_WEAPONS=1` lists each gun's muzzles
  and then the map's definitions with more than one, each found on its model or not (e05:
  `h_13e090b2 ... muzzles [f9eed613 ok, e2e787a9 ok]`).
* **Tracers and beams** (FUN_002317e0 -> FUN_002312e0, #237; play.rs `projectiles`, `beam_line`;
  ale_fx.rs `load_beam`). An instant ray carries its flight effect only when its counter is 0,
  which then restarts at h_eeb9e75a: the MK and the Kman every 3rd shot (the first among them),
  the Minigun every 4th, the rest every shot. The effect is placed once, at the muzzle, turned
  along the ray (FUN_0001d0c0's frame: its -z along the ray, its x the world's up x that, Bevy's
  `looking_to`), with the user parameter min(d, 200) / 200 for the ray's length d: where it hit,
  where a Rail stopped (§ Ricochet and pass-through), or its range when it met nothing. The
  effects are made for that: their emitters, turned -90 degrees about x, throw or carry their
  particles out along the ray (tracer.emt's at 113.6 m/s; tracer_snipe_beam.emt's offset runs
  out to 200 m in 0.38 s, rgun_beam.emt's to 100 m in 0.24 s), and their kill field (class
  h_027b6cd9, `CLASS_KILL_FIELD`: a plane facing back along the ray, past which a particle dies)
  stands 200 m x the parameter out (tracer.fld, rgun_s_kill.fld; tracer_snipe_kill.fld 179.4 m
  x it). So the MK's, Kman's and Minigun's tracer, the L-Shots' trail and the Rails' beam end
  where the ray did. Fired from the scoped eye, the beam starts 0.25 m right of it and 0.1 m
  below and is aimed at the ray's end, so in the scope it comes in from the lower right. A beam's
  appearances turn by their facing type as the game's vertex shader does (0x3b8bf8: facing 3
  along the motion, the tracer's streak and the Rails' side arcs, which stand on their particles,
  shape 3; facing 2 square to it, the L-Shots' rings and smoke cards), and its particles start
  where the game's spawn for their emitter puts them (`Compiled::game_spawn`, #67: the Rails'
  rings, rgun_perp_s.emt, a game cube whose height grows 116 m a second with OffsetHeight set
  (code-map 0x64e30), are strewn along the ray from the muzzle out, where the demo's spawn had
  them all within a few metres of the gun, a tube of rings round the camera). Before, the effect was
  spun so its emitters' own turn threw it down into the ground at the muzzle, a tracer's
  camera-facing streak could cross the whole scoped view, and nothing stopped it at the hit (and
  before #115 it flew straight up, the thin vertical line over the Minigun). Checked against the
  takes:
  * The MK on the test map (`-- --test BF_NO_SQUAD=1 BF_TEST_WEAPON=MK-ASLT BF_TEST_GOTO=0,-8,0,-8
    BF_CAMERA_PITCH=-0.12 BF_TEST_FIRE=hip BF_SHOT_LOG=1`, the pillar 10 m out): on every 3rd
    round a short orange streak at the strike end, gone into the pillar within 0.1 s, as #63
    take01 shows it (f325-f328 of the take at 60 fps: the orange streak below the reticle,
    shrinking into it); the Minigun's on every 4th the same (`BF_TEST_WEAPON="RVG50 Minigun"`).
    Scoped (`BF_TEST_FIRE=1`) it comes in from the right to the reticle. The phase: a new weapon
    object's counter starts at 0 (FUN_0022d330 zeroes +0x290 at 0x22d419; only FUN_002317e0
    writes it after), so a fresh gun's first round carries the tracer, as the demo's does; #63
    take01's first tracer on its second tap is the snapshot's counter (its MK had fired 3k + 2
    rounds before it, the only start the code allows), not a different rule (#287).
  * The L-Shot-75 at e40's cliff 25 m out (#62 take05's spot: `BF_MAP=e40 BF_CHARACTER=1
    BF_TEST_WEAPON=L-Shot-75 BF_TEST_GOTO=-16.16,152.62,-16.66,152.52 BF_CAMERA_PITCH=-0.05
    BF_TEST_TRIGGER=2.0-2.08`, 27 m to the hit): a thin pale line from beside Flint's head to just
    short of the reticle for ~0.15 s, then a grey smoke trail along it, gone by ~0.5 s, as take05
    f168-f196. Scoped at level 2 (`BF_TEST_SCOPE=2,1.5 BF_TEST_TRIGGER=3.0-3.08`): the line from
    the lower right to short of the reticle, with white rings along it, as take12 f195-f209.
  * The Rail RVR at e34's gate 10 m out (#67 take01's spot: `BF_MAP=e34 BF_CHARACTER=2
    BF_TEST_WEAPON="Rail RVR" BF_TEST_GOTO=-52.76,31.02,-52.9,31.5 BF_CAMERA_PITCH=-0.08
    BF_TEST_TRIGGER=2.0-2.08`; for the HUD on e34 add `BF_NO_SQUAD=1
    BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`, the start's letterbox off, camera.md, the
    tester's): the spark at the hit, red-violet arcs from it down toward the gun
    turning blue over ~0.15 s, then blue sparks at the hit, as take01 f19-f41. The ray goes
    through the gate and stops at a wall 38.3 m out, the beam's length.
  * Not matched: in the L-Shot's scope the demo draws more rings than take12's three or four,
    and a smoke cloud far denser than its faint haze; the Rails' rings make a bulb round the
    shooter's head where #67 take01 / take06 show a thin core with a few rings farther out (#287,
    still open). What #287 traced:
    * The count and its spread are the code's, and the demo's: FUN_00065800 cuts each update's
      count (trunc(elapsed x Emitter_Frequency), the emit count's keys passed, x the LOD curve)
      and spreads the batch over the emitter's move since its last batch, in place and age
      (0x65a73-0x65aa7 the move over the count, 0x65ca2-0x65ce5 each particle stepped back by it
      and its age by elapsed / count), all at the emitter's point when the particle system's frame
      is the emitter's own (0x65c3a-0x65c8b zero the steps). rgun_perp_s.emt: 16 rings a game
      frame for 0.21 s, strewn by the cube's spawn (0x64e30) over [0, 2 x Height) along the ray,
      Height growing 116 m a second; the demo's count (its rate, a render frame at a time) gives
      the same 16 at 30 fps (`BF_ALE_LOG`).
    * A curve's ends (FUN_002dfc10): its flags' low four bits say what it does before its first
      key (0 holds it) and the next four after its last (1 repeats the keys' span from the first
      key, 2 repeats it adding the change, 3 runs it back and forth, 4 goes on along the end key's
      tangent); between keys it's a Hermite curve from their tangents (FUN_002df650,
      FUN_002df730). The demo repeats a flagged curve from 0 over its last key's time
      (`Curve::at`) and runs straight between keys. rgun_perp_s's life (48: back and forth) and
      tracer_snipe's (16: repeat) come within a few hundredths of a second either way: not the
      difference.
    * What differs: seen from behind the shooter, the demo's rings near the gun (0.2-1 m across,
      3-5 m from the camera) bulge round the right of Hawk's head. The bulb is the rings alone:
      drawn without rgun_perp_s's pair (a local build), the side arcs and the core come up from
      Hawk's helmet to the reticle, red turning blue, as #67 take06 f013-f019; drawn without the
      arcs, the bulb stays. At take06's spot (`-- --test BF_MAP=e40 BF_CHARACTER=2
      BF_TEST_WEAPON="Rail RVR;Foley 356 Tact" BF_TEST_GOTO=-16.2,152.68,-16.69,152.58
      BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=2.0-2.08`, frames 65-68 at 30 fps)
      the demo draws a dozen rings or more, up to ~100 px across beside the helmet, where take06
      shows five or six, ~15 px by the reticle to ~45 px just over the helmet (f011-f012, both
      1280 x 720). The camera differs a little too: at the shot it's still closing in on the
      raised gun (the boom 2.96 m to 1.24 m over the frames round it, `BF_CAMERA_LOG`), the
      helmet ~116 px across where take06's is ~100. #287 take05's side view shows no ring (the
      game fades a quad seen within 20 degrees of edge-on, FUN_002a1a50's AngleFadeOut, which the
      demo doesn't draw) and nothing within ~3.5 m of the muzzle after the first game frame: the
      side arcs' near end 3.2 m out in the first, 5.3 m in the second, 5.8 m in the third (frames
      41-46 at 60 fps). What takes the near rings away isn't found:
      * rgun-mkill, the kill plane that sweeps out from the muzzle, is too slow by its curve (0.31
        m behind the muzzle to 8.7 m out over 0.04-0.46 s), and the arcs' lives (0.17-0.21 s)
        outlast it.
      * The spawn asks the appearance's fields first (its vtable +0xb4, 0x60f90, through each
        field's +0xac, the kill field's 0x66650): a particle a kill plane would take isn't made.
        rgun-mkill starts behind the muzzle, so that doesn't thin the near rings either, and the
        add (+0xb8, 0x2a14b0) draws from a pool that grows: no cap on the count.
      * The emitter's time: the game steps it before it reads a curve (FUN_00065800 calls the
        node's step, 0x2a5b10, first), so a beam's first batch is read one game frame in, where
        the demo's beams read theirs at the frame's start. Tried for the beams: the first rings
        spread over 14.2 m instead of 6.4 m and the first arc starts 3.4 m out (take05's 3.2 m),
        but the bulb stays, and with the demo's own count for beams (no EmitCount keys: the
        game's first batch of rgun_beam.emt is three points, its two keys and one by its rate,
        where the demo's is one, then 12.5 m out) the core and the arcs start well above the
        helmet where take06's come up from it. Not applied.
      * So it's a code question (the analyst); the L-Shot's smoke along the line to the reticle,
        where take12's gathers at the lower right near the gun, may share it.
* **The Minigun's spin** (#68; `src/bin/play_minigun.rs`): its item set is IFSET_MINIGUN
  (function-type 16), whose per-frame hook FUN_00147f90 (vtable 0x39b8f8, the item-set factory
  0x146920; the analyst's reading, todo/weapons-plan/answers-batch1.md §12) keeps a spin level:
  1 at once while the gun's cooldown runs with its fire flag set, else falling 1.0 a second
  (FUN_00147e50). At 1 the loop h_0e8038d2 starts, once, fading in over 0.1 s (FUN_00147bc0,
  FUN_00147c20: 2D for the player's holder, at the gun for another's); below 1 the spin-down
  h_f00b38e4 plays once in its place (FUN_00147d70), and the loop can't start again until that
  ends (its flags clear then, FUN_00147b90: a reading). Both are in the common bank. The item's
  attached effect while it spins (FUN_00147ed0) is empty for the Minigun. Holstered, switched or
  dropped, it all stops (FUN_00148030). The barrel turns at 28 rad/s x the level (the demo's
  rate; the game's barrel turn isn't traced), where the demo eased it up and down on its own
  before; the spin starts with the first round (the demo's raise before it is held in the gun's
  cooldown, which the game's isn't). Nothing changes the rate: 15 a second from the first round,
  no spin-up (#68 take01).
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
map's data has it (the test map has every hand weapon; a label two definitions share takes the
one that isn't WTYPE_DATA, so `"iKhan-GPL"` is the gun h_e9be70f0 and never #147's vaporise
blast, #55); a list, `<a>;<b>`, fills the slots in
order from the first (#35's Tex: `"Confed LZR-50;Jax-iC"`). On a level whose spawn-triggers start
the squad (e34, e40, the tutorial: #220) the character carries its spawn's template weapons, and
the hook now sets those too (#72): it did nothing there before. `BF_SHOT_LOG=1` now also prints each
pellet (its hand, frame, spread, walk, tracer), the gun's accuracy and, last, who fired it (as
the combat log names it, #227); `BF_TEST_FIRE=hip` fires
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
  walking up to ~15 degrees from the first; holes on the pillar and on sdm_e34's walls. The
  world-frame walk (#49; `BF_TEST_WEAPON="Bower 20" BF_SHOT_LOG=1`, the trigger held, each shell's
  six directions from the log): across, 9.3 degrees at every heading (24 shells each way; over 77
  facing +x 9.45, sd 4.1, the quantiles p10-p75 within 0.3 degree of the rule's 5.3 / 6.5 / 8.7 /
  11.2); up and down 0.8-0.9 facing +x (the rule 0.7), 9.6 facing +z (9.2), 2.2 at e40's 25 m spot
  (take09's, `BF_MAP=e40 BF_TEST_GOTO=-15.2,152.3,-15.69,152.21`, heading -167: the rule 1.7 at
  -170, the take 0.8); the cadence, the round per shell and the reload as before. (Its
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
* **The compare round** (#63 MK-ASLT, #54 Foley 356 Tact, #68 RVG50 Minigun, #50 / #51 / #52 the
  Confed LZRs; their combined spec `todo/63-mk-aslt/spec.md`), on the test map (`-- --test
  BF_NO_SQUAD=1 BF_TEST_GOTO=0,-8,0,-8 BF_CAMERA_PITCH=-0.12`, held fire from `BF_TEST_CLIP`;
  `BF_SHOT_LOG`, `BF_AMMO_LOG`, `BF_SOUND_LOG`):
  * Cadence: the MK 4 frames a round (7.5 a second; the takes 6.9-7.3, xemu's slower frames),
    the Foley 10-11 (2.89; the takes 2.79-2.93), the Minigun 2-3 (12.3; the takes 11.1-11.8), the
    LZR-10 16 (1.88; 1.76-1.90), the LZR-23 12-13 (2.45; 2.34-2.49) with one shot a cycle (its
    burst count 1, the delay 5 unused), the LZR-50 8-9 (3.53; the takes' mean 3.53). A tracer on
    the MK's every 3rd round, the Minigun's every 4th, the Foley's every round; a bolt on every
    LZR shot.
  * Reload (the clip set small, the trigger held): the reserve drops 0.07-0.14 s after the last
    round (the cooldown's end; the takes 0.1) for the MK and the Minigun, 0.33 s for the Foley
    (0.4), and the clip is full 1.00 s after that, the magazine-in of Tex's and Hawk's reload
    clips (the takes: Hawk 1.0; Tex 1.1 and 1.2, within the 10 Hz log's 0.1 s).
  * The LZRs recharge a round every 2.00, 1.27 and 1.00 s (the takes 2.006, 1.266, 1.0: 1.25 s
    lands on the 30 Hz frame after it).
  * Damage, on e40's Shrike Soldier frozen 9 m out (`BF_MAP=e40 BF_TEST_GOTO=-17.21,152.02,-17.70,151.93
    BF_TEST_NPC='0.5,shrike soldier@e40,enemy,frozen,-26.08,150.46' BF_TEST_NPC_HP=1.0,1,300`,
    `BF_COMBAT_LOG`): every hit of the six guns (78) inside the roll x the part's factor x 1.4.
    The takes' m03_a trooper (BALLISTIC x0.8) isn't loadable on the test map.
  * The Minigun's spin (`BF_SOUND_LOG`): the loop with the first round, the spin-down 0.1 s after
    the release, a tap's round spinning it too.
  * The scopes (the MK's 2, the Foley's 3: one level each for everyone but Flint): § The scope.
  * Not done: the Foley's silence (its noise radius 10: no AI hearing yet); the MK's scoped held
    rate (no take). (The LZR bolts fly since #136: § Bolts; the tracers and beams are drawn as
    the game places them since #237: **Tracers and beams** above.)
* **The Kman Auto** (#60; `todo/60-kman-auto/spec.md`), Hawk (`BF_CHARACTER=2`) with it: nothing of
  its own to build. Held fire 5-6 game frames a round, 5.38 a second (the takes 5.18-5.62: 6 a
  second is exactly 5 frames, so the frame-time wobble splits 5 and 6), the same scoped (its
  scoped rate is also 6); the accuracy from 90 down 10 a round to its floor of 40; a tracer on
  the first round and every 3rd. From a clip of 3 (`BF_TEST_CLIP=3`, e40's take05 spot) the
  reserve drops 600 -> 565 0.2 s after the last round (take02: +0.2), the clip is full at her
  light reload clip's magazine-in 1.0 s later (the Foley's run above times it by its reload sound;
  take02: +1.0 s), and she fires again 1.33 s after the drop, at the clip's end. Its scope (§ The
  scope): one level for Hawk's skill, 2x, the frame with the gun's 32.7-unit crosshair in the
  middle, in in 0.15 s; an exit press 0.3 s after entering waits for the lockout; the shared
  in / out sounds 135ec2ce / f9ffc972. On e40's Shrike Soldier frozen 20 m out (frozen, 300 HP)
  14 hits of 22 rounds: x1 parts 2.73-5.53 (the rule 2.45-6.65), forearms and calves 1.33-2.99
  (1.22-3.32); take08's at 25 m were 2.0-6.3, none on a forearm. Its tracer is the MK's
  (§ Tracers and beams, #237). Not done: hits at its 50 m range and scoped hits (no take with the
  aim verified).

### Bolts and their hits: the sweep, hit effects, lights and sounds (#57)

`src/bin/play_bolts.rs` (the flight and its sweeps, #136), `src/bin/play.rs` (`world_hit`,
`hit_effect_at`, `start_flight`, `land_player_shot`, `land_ai_shot`), `src/bin/play_fx.rs`
(`spawn_fx`), `src/ale_fx.rs` (`load_flight`) and `src/arena.rs` (`ray_shot`, `ray_material`),
from default.xbe's bolt pool (FUN_0022bac0, FUN_00225220, FUN_00225440) and the damage
handler's tail (0x22d0b0 -> FUN_0022cd80, FUN_0022b5b0); the analyst's answers are
todo/weapons-plan/answers-batch1.md (group A), the takes and their measures todo/57-jax-ic/.
The bolt guns of weapons batch 1 (the Jax-iC, -iP, -iR, the Saryls, the Bioreactives) and the
squad's LZRs and cutter go this way.

* **The launch** (#136; FUN_0022bac0, play_bolts.rs `launch_bolts`). A flying shot of bullet-type
  0 or 3 (under the demo's `INSTANT_SHOT` speed) is a bolt (`Shot::bolt`), FUN_0022bac0's pooled
  object: a moving object from the muzzle along the shot at the bullet's speed, living the
  bullet's range in the holder's hands over its speed (FUN_002324c0), its damage roll made at the
  launch (FUN_0022c940: Damage min..max from the game's random state, `GameRandom`, x the
  shooter's skill's factor then) and kept as the record of every hit it makes.
* **The sweep** (#136; FUN_00225220 -> FUN_00225440, play_bolts.rs `fly_bolts`, `Bolt::advance`).
  Each 30 Hz game frame its life runs down (out: it ends where it is) and its byte +3 counts down
  from 3; not yet 0 it moves on unchecked, at 0 a ray is swept from its last checked point over the
  way since and this frame's step: 8 m at 80 m/s, 10 m at 100 m/s. So a hit lands k x 0.1 s after
  the shot, on the first sweep that meets it, as the takes count it (from the frame after the
  shot's; the code would count the launch frame as the first, a frame sooner: answers-batch1.md
  §1 leaves it to the takes). The sweep meets what's there then (`sweep`): the world as the
  weapons' ray filter takes it (`Arena::ray_shot`: the terrain and the placed objects, a breakable
  among them, not the level's invisible blockers, which the filter's PID mask 0xfffffff8 leaves
  out), a character where it is now (standing, its cylinder; knocked down, its ragdoll's bones; a
  corpse, its ragdoll), a Sentry; never its shooter. On a hit the bolt is set back 0.1 % of the
  way toward its last checked point, the hit is dealt that frame (FUN_00224770 -> 0x22d0b0: the
  damage through `land_player_shot` / `land_ai_shot`, the hole, the hit effect and its sound, a
  breakable's roll through `ObjectHits`, a Sentry set off, a body on the ground shoved) and the
  bolt ends. Before #136 the demo picked a bolt's hit at its shot (the body or the world on its
  line then) and landed it on the sweep that would reach it: a target that stepped out of the way
  was still hit, one that stepped in wasn't. The loose pickups a sweep passes are knocked as by a
  shot's line (`Sweeps`, play_pickups.rs).
* **The flight.** A bolt moves from its first frame (before, it waited two at the muzzle): at
  the muzzle in the shot's frame, then on by its speed each game frame (drawn on by the frame's
  time not yet run in game frames), on into what it struck until the sweep that finds it (the
  Jax-iR's ball gone into the wall before its burst, todo/59 take01); one that meets nothing ends
  with its life, without the sweeps after it (the Jax-iC's last reaches 56 m of its 60). Its
  flight effect rides it (`start_flight`, compiled by `load_flight`): the effect's emitters ride
  with the bolt and their particles go with it when it ends (the ball, its halo), except an emitter
  under the library's world node (h_ee223b51, node flag 1) whose appearance hangs at the root,
  which leaves its particles in the world (the trails). The Jax-iR's ball hangs under the node
  with its appearance and rides (#59: read as a trail it lay a violet band along the whole way,
  where take01 shows nothing between the gun and the wall).
  Where an emitter has the demo's ATTACHED (h_e2999ffd: the LZRs' bolts) it still decides,
  though code-map's 0x64e30 reads that parameter as the cube emitter's offset height. The rule
  is inferred from the effects' trees and the takes (the balls gone the frame their bolt ends,
  trails fading where they were laid), not from code. Before, a root emitter without the flag
  left its particles in the world, and a bolt's effect began emitting a frame after it.
* **A hit on the world** (`world_hit`): the struck triangle's world material
  (`Arena::ray_material`: the collision's terrain comes as twin triangles, and the one whose
  front faces the shot carries the material and flags, the back twin 0) and the shot's ammo-type
  pick from world-materials, as FUN_0022cd80 and FUN_0022b5b0 do:
  * the hit effect: the material row's for the ammo-type (h_f6f08598; only the flesh materials
    have one), else the weapon's own (h_08d28037) at user parameter 1.0;
  * the material's debris effect (`debris-effect`: wmat_dirtpuff's tan puff on e34's ground,
    wmat_concrete's grey dust ...) at user parameter 0.036 x the hit record's damage, at most 0.9,
    1.0 for a killing hit (`hit_sp`; the record's damage is the roll x the falloff here);
  * one of the row's four hit sounds at random (FUN_0022b5b0: the random state's low 2 bits),
    from the hit by the bank's distances (`distance_gain`: the sonic rows full within 9 m,
    silent beyond 20 m). Before, a world hit had no sound and no debris.
  * The roll and the sound's pick are draws of the demo's copy of the game's one random state
    (`GameRandom`: the LCG at 0x46cfd8, FUN_0022c940's roll from its bits 9-31), drawn per hit;
    the hit effects' and each shot's flight effects' seeds are draws of it too (the demo's). The
    game picks the sound by the state's low 2 bits, which such an LCG steps through with a
    period of 4; there the rest of the game's draws mix them, here each shot draws the same count
    and the low bits would pick one sound for good, so the demo picks by the top 2 of a draw
    stirred by an integer hash (`GameRandom::stirred`; the raw top bits clustered too: a lone
    shooter's first five hits drew one sound). A seed from the muzzle's place stood in at first
    and saturated to 0 wherever its sum was negative: every hit at e34's gate had one sound, one
    roll and one burst.
  * Each effect in its alignment's frame (`hit_frame`; the effect type's `alignment-type` per
    alchemy effect, FUN_0021a850, the ALIGN_TO_ table at 0x3bd298): 0 the shot's way (the frame
    FUN_0001d0c0 builds: its -z along the way), 1 that way reflected off the surface, 2 the
    surface's normal, outward, 3 the world's axes. The hits' emitters fire along their -z (they
    are turned -90 degrees about x), so an alignment-2 burst (the sonic's, the particles')
    comes out of the surface and an alignment-0 one goes on along the shot. Before, every hit
    effect had its +y turned back along the shot, which threw those emitters across it.
  * Each effect type's light-effect is a point light at the hit (the hit lights:
    light_hit_sonic's cyan wash, #57), as the muzzle lights and the blasts' are played. The
    debris types have none.
  * The effects are compiled with the game's emission count, shapes and facing 3
    (`load_recorded_facing`), with the type's bounding sphere for the distance LOD.
* **The bullet's explosion** (#53; play.rs `shot_explosion`): before the struck material's hit
  effect, the damage handler plays the bullet's own effect type at every direct hit, on the world
  or a body (0x22d0b0 -> FUN_0022cfc0 -> FUN_0022cbe0: h_ec23d593, the weapon type's +0x250, at
  user parameter 1.0, with its light) and its impact sound (h_e5618348, +0x254). Of the hand guns'
  shots only the two cutters' come here with one: cutter_hit and plut_cutter_hit with
  light_hit_cutter, the white star where a blade strikes (#53 take01 f17-f18 of 5.20 s on, on the
  tower; take03, on the enemy's chest); every other gun names its hit in h_08d28037 instead. The
  iKhan-GPL names one too (exp_mirvgrenade), which its round, a plain shot after batch 2, played
  here (#287); since #239 the round is a lobbed missile and plays it where it goes off
  (**Missiles** below). A bolt's hit on a body, a corpse or a Sentry plays it from `deal`
  (play_bolts.rs), a world hit from `world_hit`. Not
  done: the per-material overrides at +0x25c, taken when the hit record's +0x58 bit 1 is set (not
  traced for a bolt). Before, the cutters' hits had only the material's debris.
* **A hit on a body** (play_fx.rs `spawn_fx`): the flesh row's effect for the ammo-type, when the
  demo doesn't make it by hand (the squad's ballistic, cutter and laser ones), is played from
  the data with its light: the sonic h_177a62d8 (wmat_fleshit_sonic), the particle h_ed6363bd,
  the bio h_f8d4e76d, the rail's and the energy's. Its user parameter is the hit record's
  (`damage::Hit::record`: the roll x the skill's factor x the falloff x the part struck x the own
  side's 0.7, which 0x22d0b0 hands to FUN_0022cd80; the victim's step and type factor come
  after), 1.0 for a killing hit; aligned as above, the body's normal taken out from the hit
  cylinder's axis (the demo's stand-in for the struck part's collision).

Test hooks: `BF_SHOT_LOG=1` also prints the map's world materials as it loads ("world
materials (id: triangles) 0: 15482 terrain + 4596, ..."), each world hit ("t=2.37 hit on world
material 5 at [...] 8.4 m out (ammo-type 3): effect h_fed64261 sp 1.00 light h_eb110330 (the
weapon's), debris h_fe578f8f sp 0.79, sound h_0655c729 vol 0.57 (slot 1)": which of the row's
four sounds the random state picked; a bolt's says how far along its sweep ("10.3 m along its
sweep") where a shot's says how far out; a cutter's adds "explosion h_0557b326 light h_16d39d8b" before its effect) and each
hole's life; and each bolt's launch ("t=2.20 bolt 1 launched by tex from [...]
along [...] at 80 m/s: roll 21.97 x skill 1.00, life 0.750 s (range 60.0 m)"), each sweep that
meets something ("t=2.50 bolt 1 sweep at game frame 9 (0.30 s): hits Shrike Soldier (NPC 1) 7.4
m on from its last checked point 23.4 m out", with "(past a level blocker 0.40 m on)" where the
sweep went by one) and its end at its life's ("bolt 1 (Jax-iC) ends at its life's end after 23
game frames, 58.7 m from its muzzle"). `BF_COMBAT_LOG=1` each flesh hit effect played from the
data ("flesh hit effect h_177a62d8 at [...] sp 0.41 light h_eb110330").

**Moving targets** (#136; the test map, `-- --test BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0
BF_TEST_WEAPON=Jax-iC BF_TEST_GOTO=4.5,20,4.5,20 BF_CAMERA_PITCH=-0.115 BF_TEST_NPC_HP=1.0,1,300
BF_SHOT_LOG=1 BF_COMBAT_LOG=1 BF_CAPTURE_FPS=30`, a dm Shrike Soldier 25 m out):
* Standing (`BF_TEST_NPC="0.3,shrike soldier@dm,enemy,dummy,4.5,-5" BF_TEST_TRIGGER=2.0-2.08`):
  the bolt fired at 2.20 meets it on its sweep at game frame 9 (0.30 s), 23.4 m out.
* Stepping aside (the same with `BF_TEST_NPC_MOTION="2.23,1,dash,10.5,-5"`, the dash starting a
  frame after the shot): the same bolt, along the same line, meets nothing and ends with its life
  58.7 m out; frames 66-77 show the ball pass where the Shrike stood as it dashes off to the
  right. The demo before #136 picked the hit at the shot and landed it on the Shrike.
* Crossing (`BF_TEST_NPC="0.3,shrike soldier@dm,friend,dummy,-1.5,-5"` running across the line
  from 2.2 s, `BF_TEST_NPC_MOTION="2.2,1,dash,10.5,-5" BF_TEST_TRIGGER=2.65-2.73`): fired at 2.80
  with the Shrike 3.3 m left of the line, the sweep at frame 9 (24 m) meets nothing and the one
  at frame 12 meets the Shrike 24.2 m out, where it is by then (its left upper arm, x0.7 on the
  own side: a friend); before #136 the bolt missed it.
* The pillar, e34's gate and the frozen Shrike Heavy recipes below land as before: 8.5-8.8 m out
  at frame 6 (0.2 s), 5.3 m out at frame 3 (0.1 s). A supply crate put down 6.6 m out
  (`BF_TEST_PLACE="0.5,supply crate,4.5,12,0"`) breaks on the sweep that meets it (24.38 x 5,
  SONIC), the bolt ending there; a dead Shrike's body on the ground 8 m out stops the bolts after
  it and is shoved ("hits Shrike Soldier (NPC 1)'s body"); a Sentry 9.6 m out (the grenade
  page's `BF_TEST_SHOOT_MINE=2.2`, an LZR-23 bolt) goes off on the first sweep, 0.1 s after the
  shot. `cargo test --bin bf_play bolts` checks the sweeps' frames and lengths and the life's
  end (80 m/s and 60 m: sweeps on frames 3-21, the end on 23; 100 m/s and 80 m: the end on 24).
* Not reproduced: the game sweeps against its bone bodies; the demo's characters are its
  cylinder (standing) and ragdoll bones (lying), as the shots' have been. A bolt passes a liquid's
  surface without a splash and goes on to the bed: the blast pool has no liquid code (#5,
  todo/146-liquids leads §5.1; only an instant ray splashes, levels.md "What a liquid plays").
  Whether the game's sweep stops on the liquid's body isn't checked.

Verified (`BF_CAPTURE_FPS=30`, the logs, against todo/57-jax-ic's takes and spec):
* **The Jax-iC at the test map's pillar** (`-- --test BF_TEST_WEAPON=Jax-iC BF_TEST_GOTO=0,-8,0,-8
  BF_CAMERA_PITCH=-0.12 BF_TEST_FREECAM=0.5,1.5,1.2,-13.5,18.3,-10
  BF_TEST_TRIGGER=2.0-2.08,3.6-3.68,5.0-9.0 BF_TEST_CLIP=3 BF_SHOT_LOG=1 BF_AMMO_LOG=1`): held fire
  every 15-16 game frames (the take 15.5); the reload 0.50 s after the last round (reserve 200 ->
  164) and the next round 1.30 s later (the take 0.50 and 1.30 s); each hit 0.2 s after its shot
  8.5 m out, the weapon's sonic burst with its light washing the pillar, the default material's
  debris, the hole.
* **The gate's foot on e34** (take01: `-- --test BF_MAP=e34 BF_TEST_WEAPON=Jax-iC
  BF_TEST_GOTO=-55.2,31.3,-55.2,31.8 BF_CAMERA_PITCH=-0.26
  BF_TEST_TRIGGER=2.0-2.08,4.2-4.28,6.4-6.48 BF_SHOT_LOG=1`): the ground is material 5 (DIRT):
  the hits 8.4 m out land 0.2 s after the shot and the ones 7.2-7.4 m out 0.1 s after (the take
  T0 + 0.2 s at ~10 m), with wmat_dirtpuff's tan puff after the burst and one of the row's
  sounds (h_0655c729; the take's h_0f52f7bf and h_1d5c9693 are two of the same four).
* **A frozen e34 Shrike Heavy 7 m out** (take03: the same with `BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC="0.3,shrike heavy,enemy,frozen,-55.39,38.3" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_TRIGGER=2.0-2.08,4.2-4.28,6.0-12.0 BF_COMBAT_LOG=1`): every hit 0.1 s after its shot;
  29.6-35.1 on its right calf (roll x0.5 x STANDARD 1.4 x SONIC 2: the take 30.5-34.9 on its
  hovering legs), dead on the 10th hit as in the take; each hit plays wmat_fleshit_sonic with
  light_hit_sonic at user parameter 0.38-0.45 (0.036 x the record's 10.6-12.5), 1.00 on the
  kill. At `BF_CAMERA_PITCH=-0.12` the hits go to a thigh: 61.0-68.3 (the takes' standing
  Shrike 59.5-68.8).
* Against the takes' pictures the burst's core is about the take's (14-16 units at ~10 m) but
  paler, and the take's violet wash of the light over the gate's foot (80-140 units wide) is
  missing on e34's lit surfaces; on the test map's pillar the light shows. The light's reach on
  the level's own surfaces is #143's.
* **The Jax-iP** (#58, todo/58-jax-ip): at the pillar (`BF_NO_SQUAD=1` and the recipe above with
  `BF_TEST_WEAPON=Jax-iP`): held fire every 20-21 game frames (the take 19.8), the reload 0.67 s
  after the last round (the take 0.71), the hits 9.4 m out 0.2 s after their shots, sonic_proj_hit's
  white core with light_hit_sonic. On a frozen e40 Shrike Heavy (`-- --test BF_NO_SQUAD=1
  BF_TEST_INSTANT_KILL=0 BF_TEST_WEAPON=Jax-iP BF_TEST_GOTO=4.5,20,4.5,20 BF_CAMERA_PITCH=-0.08
  BF_TEST_NPC="0.3,shrike heavy@e40,enemy,frozen,4.5,-5" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_TRIGGER=2.0-2.08,3.6-3.68,5.0-9.0 BF_COMBAT_LOG=1`): 25 m out every hit lands 0.4 s
  after its shot (take07 0.4, 0.4, 0.4), 37.0-39.0 on the body and 208.0 on the head (roll x
  SONIC 2 x 1.4: take07 35.1-38.5); 9 m out (`...,4.5,11`) 0.2 s (take03's 0.2 in 5 of 8),
  34.4-39.0 on the body, 208.0 on the head, 30.3 on an upper arm; 45 m out (`...,4.5,-25`,
  `BF_CAMERA_PITCH=-0.115`) nothing is hit, the bolt ending in the air at its 40 m (take08: no
  hit), the crosshair blue there and red at 25 m (the 28 m lock reach). The test map's own
  "shrike heavy" is the dm Heavy, whose SONIC is x1 and whose stamina shield takes the first
  40: use `@e40` (or `@e34`) for the campaign's x2.
* **The Jax-iR** (#59, todo/59-jax-ir): at the pillar, held fire every 25-26 game frames (the
  take 24.8), the reload 0.83 s after the last round (the take 0.81; reserve 200 -> 176), the
  hits 8.9 m out 0.2 s after their shots. At e34's gate as take01 (Brutus carries it there:
  `BF_CHARACTER=0` with the gate recipe above; e34's squad keeps its own loadout, so
  `BF_TEST_WEAPON` isn't read): the ball goes into the ground at the gate's foot a frame or two
  before the burst at +0.2 s and nothing is left along its way, as the take shows (`ride`, the
  trail rule above; before it a violet band lay from Brutus to the wall for 0.6 s). On the
  frozen e34 Shrike Heavy 7 m out (the take03 recipe above, `BF_CHARACTER=0
  BF_CAMERA_PITCH=-0.12`): every hit 0.1 s after its shot, 88.9-95.1 on the body (roll x 1.4 x
  SONIC 2: take03 86.4-88.2, the range 86.1-95.5), wmat_fleshit_sonic at 0.90 (the cap), dead on
  the 4th. On the test map's clear line (the Jax-iP's recipe with `BF_TEST_WEAPON=Jax-iR`): 25 m
  out 0.3 s (the rifle's muzzle 0.4 m further out puts the body at 23.8 m, inside tick 9's 24 m:
  take08 0.4 s, the takes' 25 m at 80 m/s 0.3 s in 6 of 24), 86.8-90.6 on the body; 45 m out
  0.6 s, on the calves at `BF_CAMERA_PITCH=-0.115`.
  * Not reproduced: take07's glow at 65 m lasting to T0 + 1.50-1.55 s, 0.4 s past the bolt's
    1.125 s life; the demo's ball goes when the bolt does.
* **The Saryl-45** (#70, todo/70-saryl-45; the particle family's look): at the pillar, held fire
  every 12-13 game frames (the take 12.1), the reload 0.40 s after the last round (the take
  0.40; reserve 200 -> 170), the hits 8.9 m out 0.1 s after their shots (100 m/s: 10 m a
  step). From behind Tex (`BF_TEST_FREECAM=0.5,0.6,2.2,-4.5,0,-8`) the comet's white head rises
  from the gun with its pale-blue tail down to the muzzle (plasma_bullet riding, its trail left
  in the world) and ends in plasma_hit's white star with blue shards out of the face and
  light_hit_particle's wash, the muzzle's blue glow at Tex's hand meanwhile, as take01 shows.
  Seen from 4.5 m the star's core is ~0.5 m and the shards ~1.4 m across (take01's 16-unit
  core at ~13 m is ~0.65 m), so the data's 3.74 m end size (the spec's open point) reads right:
  the quads fade out as they grow. On a frozen e40 Shrike Heavy (the Jax-iP's recipe, the
  Saryl-45): 7 m out 0.1 s, 25 m out 0.3 s (take08's 10 of 10), 45 m out 0.5 s; 18.1-20.8 on the
  body, 15.1-17.4 on an upper arm, 96.8 and 103.8 on the head, 9.0-10.7 on the calves (PARTICLE
  x1: take08 18.1-22.0, take09 106.8 on the head, take03 8.9-11.0 on the hovering Shrike's
  calves); each hit plays wmat_fleshit_particle (h_ed6363bd) with light_hit_particle.
* **The Saryl-23** (#69, todo/69-saryl-23): Hawk carries it on e34 (`BF_CHARACTER=2` with the
  gate recipe above). At the gate's foot as take01: each hit 0.1 s after its shot 7.7-8.8 m
  out, plasma_hit_sml's small white star with blue shards out of the ground, then
  wmat_dirtpuff's tan dust and one of the row's sounds (h_0655c729), the comet's head with its
  tail on the way; the star shows on the hit's next frame (+0.13 s), take01's at +0.18 s. On the
  frozen e34 Shrike Heavy 7 m out (the take03 recipe, `BF_CHARACTER=2 BF_CAMERA_PITCH=-0.12`):
  held fire every 13 game frames (the rate's 12.5 rounded up; the take 12.7), every hit 0.1 s
  after its shot, 12.9-18.9 on the body (PARTICLE x1: the range 11.9-18.9; take03 12.8 and 16.9
  on the body, the rest on the hovering Shrike's calves), wmat_fleshit_particle with
  light_hit_particle on each.
* **The Saryl-99** (#71, todo/71-saryl-99; the Saryl-45's effects): Brutus's second gun on e34
  (`BF_CHARACTER=0 BF_START_WEAPON=1`). On the frozen e34 Shrike Heavy 7 m out (the take03
  recipe, `BF_CAMERA_PITCH=-0.12`): held fire every 10-11 game frames (the take 10.1), every hit
  0.1 s after its shot (take03's 17 of 17), 11.0-16.4 on the body (PARTICLE x1: take03
  11.2-15.7), wmat_fleshit_particle with light_hit_particle. On the test map's clear line (the
  Jax-iP's recipe with `BF_TEST_WEAPON=Saryl-99`) 25 m out every hit 0.3 s after its shot
  (take08's 2 of 2), 11.2-15.0 on the body, 64.3 on the head.
* **The A10 Bioreactive** (#47, todo/47-a10-bioreactive; the bio family's look): at the pillar
  (`BF_TEST_WEAPON="A10 Bioreactive"`), the hits 8.8 m out 0.2 s after their shots (take01's
  burst at T0 + 0.20 s), the weapon's own bio-flash_hit and bio-shrap_hit out of the face with
  light_hit_bio, and its green hole h_1ce6db03 living 60 s (`BF_SHOT_LOG`'s "hole ... life 60
  s, fading over its last 5 s"); the reload 0.40 s after the last round. Seen from a free camera
  standing in for the take's (`BF_TEST_FREECAM=0.5,2.5,2.0,6.0,5.5,-3.9`, 23.7 m back: chosen when
  the demo drew 45 degrees down against a guessed ~74; the game's view is 55.4 down since #269, so
  the take's own camera, about 13 m back, is the one to stand in for now) the flash's white-green
  core was ~17 units
  (take01 12-14) and its green haze thins over ~0.8 s (take01: to 10% at +1.3 s); the green
  bolt and the muzzle's green glow at Tex's gun as the take's. On a frozen e40 Shrike Heavy 7 m
  out (the Jax-iP's recipe with the A10): every hit 0.1 s after its shot, 11.0-12.8 on the body
  (BIO x1: take03 11.0-13.7), 9.0-9.6 on an upper arm, 63.6 on the head, the hit points moving
  only at the hits (no damage over time, as take06's log), wmat_fleshit_bio (h_f8d4e76d) with
  light_hit_bio on each. The bioreactive kill ("massive cellular deterioration") is #147's: the
  demo's BIO kills are plain.
* **The A50 Bioreactive** (#48, todo/48-a50-bioreactive; thin footage, no enemy or reload take):
  at the pillar (`BF_TEST_WEAPON="A50 Bioreactive"`) held fire every 10-11 game frames (take01
  10.4), the hits 8.5 m out 0.2 s after their shots with its hit type h_fe10809d and
  light_hit_bio, the green hole; its two-arc crosshair blue on the pillar, red on a Shrike 7 m
  out. On the frozen e40 Shrike Heavy 7 m out (the Jax-iP's recipe with the A50): every hit 0.1
  s after its shot, 16.9-18.5 on the body (the rules' 16.1-18.9, no take to check), 14.1-14.8 on
  an upper arm, 90.2 on the head, wmat_fleshit_bio with light_hit_bio. Not checked: the reload
  (none filmed: take02's reserve was written where the game reloads from nothing) and any
  damage against a take; the bioreactive kill is #147's.
* Regressions: the squad's eight guns at the test map's pillar (MK-ASLT, Minigun, Bower 20,
  Foley, L-Shot-50, LZR-23, LZR-50, Feral Cutter: two taps each, against main's frames, the
  free camera 4.5 m from the pillar's face): the instant rays' sminigun_hit now flashes out of
  the surface where it lay as a thin streak across the shot, with the default material's
  debrishit dust; the LZRs' and the cutter's hits land on the sweep and their bursts come out of
  the surface (the LZR-23's ~1.5 m burst, as its take01 shows); the LZRs' bolts show their red
  streak in flight in this side view, where main's showed almost nothing between the muzzle and
  the hit (no take shows a bolt from the side). The test map's NPC fights (testtools.md), the
  tutorial's first patrol (brain.md) and the Therm Sweeper at the pillar play as before.

### Ricochet and pass-through (#238)

`src/bin/play_bolts.rs` (`Bolt::after_hit`, the bolts) and `src/bin/play_rays.rs` (the instant
rays), from default.xbe's FUN_00225440 (a bolt's flight) and FUN_00230860 / FUN_00230400 /
FUN_002318e0 (an instant ray's hits); the bullet block's three values in `bf/weapon.rs`
(`ShotData::object_hits` h_1835f15e, `character_hits` h_fe4f3035, `pass_through` h_11caf57b:
FUN_0018a8d0's bullet +0x2c / +0x30 / +0x34). The data: the Feral Cutter and the PU Cutter n1 5,
n2 0, no pass; the Ion Beamer n1 5, n2 0, pass; the Rail RVR and CLVR n1 1, n2 500, pass; every
other gun 0, 0 (the Saryl-45's pass flag has nothing to spend).

* **A bolt** (FUN_00225440 after a sweep's hit, which is dealt first as any hit is): with the pass
  flag it goes through a game object that isn't a character while n1 is left (one off it), a
  character while n2 is (one off that), on from the hit point with that thing left out of its next
  sweep (+0x40); anything else ends it, the terrain always (it has no object). Without the flag,
  while n1 or n2 is left, it bounces: a hit on an object takes one off n1 (off n2 once n1 is out),
  any other hit one off n2, and its way is turned about the surface's normal (d - 2 (d.n) n); with
  the count at 0 it ends. So **the cutters' blades bounce off up to five objects** and stop on a
  character or the terrain, and **the Ion Beamer's bolt goes through up to five objects** and
  stops at the first character or the terrain. "Objects" are the ray result's kind 2 that aren't
  characters (FUN_000dd8e0: a body of PID 0): every placed object's parts (the e40 beach's
  standing stones, buildings, crates, gates), a broken object's remains, a Sentry; the demo counts
  the test floor's pillars and the test tools' pieces so too. The terrain blocks (PID 2) are kind
  1: a blade ends on e40's cliff. Each thing on the way is dealt its hit (its effects, hole,
  sound; a breakable takes the roll).
* **An instant ray** (FUN_00230860): one hit, or n1 + n2 with the pass flag, the latter through
  the multi-hit ray (FUN_00230400, nearest first, no blockers, no liquids, at most 256 hits).
  Its collector (vtable 0x39efc8, hit method 0x2b7af0) keeps every hit the ray caster hands it,
  merging none by their body (FUN_002b8ea0 keys its table by the record), so a closed object is
  met twice, where the ray goes in and where it comes out, and the terrain's floor once
  (`Arena::ray_shot_all`: each face it crosses, a face's twin and a shared edge once; the test
  floor's pillars and placed breakables likewise, play_bolts.rs `floor_all`). A character is met
  once (`units_met`: the loop passes over one it has met, 0x230bf9-0x230c14, neither dealt nor
  counted). Each hit takes one off a budget, a character not hit yet n2, anything else n1, and is
  dealt with a roll of its own (FUN_002318e0 -> 0x22d0b0); after the hit that drives a budget
  below 0 the ray stops. So **the Rails go through every character on their line and stop at the
  far side of the first object they enter**, dealing it two rolls, and never reach a character
  behind it; fired into the ground they go through the floor (one crossing) into a character
  below and on to the next surface. Before #287 a body was met once, so a Rail went through a
  crate or a gate leaf into the enemy behind it and stopped at the second wall: #287's takes show
  otherwise (below). The demo deals a Rail's hits in its shot's frame (the game's ray deals them
  as it's cast), where its other rays land theirs at INSTANT_SHOT; the shot's `dist` becomes
  where it stopped (its beam's length, § Shots). One hit (every other gun) stays play.rs's.
* Not done: a bolt's push on the loose object it ends on and a ray's push x3 (the demo's loose
  pickups are knocked by the lines, `Sweeps`); FUN_00230400's x3 on a character while the shooter
  is scoped (no pass-through gun has a scope); the squad AI's ray given up at a squadmate in the way
  (EVT 34, plan §4.2); a bolt or a ray at a liquid's surface (the game's single ray stops there,
  the demo's go on; since #5 only an instant ray splashes, levels.md "What a liquid plays").

`BF_SHOT_LOG=1` prints a bolt's counts at its launch ("n1 5 n2 0 passing through"), each bounce
("bounces off it along [0.561, 0.190, 0.806] (n1 4 n2 0 left)") and each pass ("passes through it
(n1 4 n2 0 left)"), and each of a pass-through ray's hits ("ray of Rail RVR 6.0 m out: hits Shrike
Soldier (NPC 1) (walls 1 characters 499 left)") and where it stops ("; it stops there", or "goes
on to its full range, 90.0 m"). `cargo test --bin bf_play bolts` checks the bolt's rule (five
bounces off objects and the end on the sixth, on the terrain and on a character; n2 once n1 is
out; five passes and the end at a character); `cargo test --lib multi_hit` the multi-hit ray's
crossings (a box twice, twinned or not and along a face's diagonal; once from inside it; the flat
floor's ground once).

Against #287's takes (`todo/287-weapon-shots/`, take-log.md's hit logs: every take-damage the
game dealt, in order), on e34 with Hawk (`-- --test BF_MAP=e34 BF_CHARACTER=2
BF_TEST_WEAPON="Rail RVR;Foley 356 Tact" BF_TEST_GO_AT=100 BF_TEST_INSTANT_KILL=0
BF_TEST_NPC_HP=1.0,all,300 BF_TEST_TRIGGER=3.0-3.08,5.6-5.68 BF_SHOT_LOG=1 BF_COMBAT_LOG=1
BF_SCENERY_LOG=1`; for the HUD add `BF_NO_SQUAD=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`,
the start's letterbox off, camera.md, the tester's: take01's recipe gives the same hits with them):
* **Two enemies in line** (take01: `BF_TEST_GOTO=-48.0,23.35,-49.61,30.16 BF_CAMERA_PITCH=-0.10
  BF_TEST_NPC="0.3,shrike heavy@e34,enemy,frozen,-49.61,30.16;0.3,h_e96829a0,enemy,frozen,-50.72,35.96"`):
  each ray hits the Heavy 6.0 m out, the Soldier 12.0 m out, then the base gate's leaf (breakable
  56, 40 hp) where it goes in, 17.8 m out (walls 1 -> 0), and where it comes out, 18.2 m (-> -1),
  and stops there. take01's every shot: the Heavy, the Soldier (or the Soldier alone), then the
  leaf 7f1e2a00 twice. Before #287 the demo's met the leaf once and went on to a wall 38 m out.
* **An enemy behind a crate** (take02: `BF_TEST_GOTO=-43.47,20.97,-41.89,32.36
  BF_CAMERA_PITCH=-0.16 BF_TEST_NPC="0.3,shrike heavy@e34,enemy,frozen,-41.89,32.36"`): the ray
  meets the yard's supply crate (h_e45338bd, the take's, breakable 460) 7.0 and 7.7 m out, two
  rolls (16.36, 15.79), and stops: the Heavy behind isn't hit. take02: the crate (its hp raised to
  500) took two rolls a shot, five shots, and the Heavy none. The demo's crate (25 hp) breaks on
  the first shot, and the next goes on into the Heavy.
* **An enemy behind the gate's leaf** (take03: `BF_TEST_GOTO=-52.2,34.5,-52.2,41.3
  BF_CAMERA_PITCH=-0.10 BF_TEST_NPC="0.3,shrike heavy@e34,enemy,frozen,-52.2,43.6"`): the leaf 6.2
  and 6.6 m out (14.43, 16.59), then the ray stops; the Heavy 2.3 m behind isn't hit, as take03's
  ten leaf hits and none on the Heavy. The leaf, dead at 0 hp, stays standing and blocking (#263).
* **Into the ground** (take04's spot, `BF_TEST_GOTO=-48.0,23.35,-48.99,27.64
  BF_CAMERA_PITCH=-0.45`): the floor 4.7 m out is one crossing (walls 0) and the ray goes on to its
  90 m, so a character under the floor would be hit, as take04's Heavy 2 m under it took every
  shot. The test hooks can't put one there (`BF_TEST_NPC` stands it on a floor): not run.
* take05 (the Heavy with two objects behind it, from the side) and take17 (the CLVR, a Shrike
  then the object 7f20f200 behind it twice) follow the same rule (#66).

Verified (`-- --test BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=4.5,20,4.5,20
BF_TEST_NPC_HP=1.0,all,300 BF_SHOT_LOG=1 BF_COMBAT_LOG=1 BF_CAPTURE_FPS=30`, dm Shrike Soldiers
frozen on the line):
* **Rail RVR, two Shrikes in line** (`BF_TEST_WEAPON="Rail RVR" BF_CAMERA_PITCH=-0.14
  BF_TEST_NPC="0.3,shrike soldier@dm,enemy,frozen,4.5,13;0.3,shrike soldier@dm,enemy,frozen,4.5,8"
  BF_TEST_TRIGGER=2.0-2.08,3.5-3.58`): each shot hits both, 6.0 and 11.0 m out (14.43 and 14.64 x
  their forearms 0.5 x 1.4, their stamina shields taking it), goes through the ground 67.8 m out
  (its one wall, one crossing) and on to its 90 m.
* **Rail RVR at a crate** (supply crates put down 4 and 16 m out and a Shrike 10 m out,
  `BF_TEST_PLACE="0.5,supply crate,4.5,16,0;0.5,supply crate,4.5,4,0"`, #287's rule): the first
  shot meets the near crate 2.5 and 4.2 m out (walls 0, then -1) and stops; it breaks, and the
  second goes on into the Shrike 9.1 m out and stops at the far crate's far side, 14.5 and 16.3 m.
  Before #287 the first shot went through the near crate into the Shrike.
* **Ion Beamer through two crates** (`BF_TEST_WEAPON="Ion Beamer" BF_CAMERA_PITCH=-0.2`, crates
  4 and 8 m out, Shrikes 14 and 19 m out): the bolt passes the first on its sweep at frame
  3 (n1 5 -> 4) and the second at frame 6 (-> 3), each taking the launch roll 14.44, and stops at
  the first Shrike 12.8 m out at frame 9; the second Shrike isn't touched.
* **Feral Cutter off a pillar** (`BF_TEST_WEAPON="Feral Cutter" BF_TEST_GOTO=4.5,-3,5.1,-3.8
  BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=2.0-2.08,3.5-3.58`): the blade meets the pillar at (9,
  -9)'s face 5.9 m out on its first sweep and bounces, [0.561, 0.190, -0.806] -> [0.561, 0.190,
  0.806]; frames 66-75 show its ribbon bend at the pillar and go off to the right. It flies on to
  its life's end (19 frames, 75 / 120 m/s).
* **PU Cutter on e40** (Brutus, `BF_MAP=e40 BF_CHARACTER=0 BF_TEST_WEAPON="PU Cutter"`): from 10 m
  (take01's spot, `BF_TEST_GOTO=-31.57,152.2,-32.02,151.98 BF_CAMERA_PITCH=-0.02`) every blade ends
  on the cliff, the terrain (material 5), with no bounce, as take01 shows none; held fire one
  every 8 frames (3.75 a second; the take ~3.8). From take05's 45 m spot with the crosshair on the
  standing stone (`BF_TEST_GOTO=0.01,167.27,-0.46,167.10 BF_CAMERA_PITCH=-0.10`) the blades bounce
  off it (an object, material 8) up and to either side, as take05's streaks leave the stone at
  10.2 s; in the demo the stone is 23.5 m out where take05's was ~40 m (the take's spot isn't
  reproduced exactly).

### The guns of weapons batch 2 against their takes (#53, #65, #67)

No tester's spec was written for these; each was measured here from its takes (`todo/<n>-*/`,
the frames pulled with ffmpeg at 60 fps, the HUD's counts and the watch logs in each notes.md).
The takes' spots are reproduced on their levels with the test map's tools (`-- --test`), since a
campaign level whose squad spawn-triggers fire at once (m03_a) moves the character from a
`BF_TEST_GOTO` start: `BF_TEST_FREECAM` straight down over the spot, `BF_TEST_TELEPORT` there, and
`BF_TEST_GO_AT=100` so the autopilot only aims (from the GOTO's start toward its goal).

**Feral Cutter** (#53, Brutus; `-- --test BF_MAP=m03_a BF_CHARACTER=0 BF_TEST_WEAPON="Feral Cutter"
BF_TEST_GOTO=-182.6,-43.5,-186.90,-52.59 BF_TEST_GO_AT=100
BF_TEST_FREECAM="0.3,-182.6,45,-41.88,0,-89.9" BF_TEST_TELEPORT=0.5 BF_TEST_FREECAM_OFF=0.7
BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=3.0-3.08 BF_SHOT_LOG=1`, take01's spot, the stone tower 10 m
out; the goal is #287 take07's first hit, where its blade left along bearing -115, 9 degrees right
of Brutus's facing -124, which the recipe aimed at before #287). m03_a's squad spawn moves Brutus
some 80 m from the GOTO's start, so every m03_a recipe here needs the three free-camera hooks: the
camera straight down 45 m up over its spot at (x, z + 1.62), where the crosshair's ray, 6 degrees
above the view's axis, meets the ground at (x, z); the teleport puts him there.
* Each blade strikes the tower (an object, material 3) on its first sweep, 0.1 s after the shot
  (take01: the HUD's 29 -> 28 at f11 of 5.20 s on, the white star at f17), with the white star of
  its explosion and sparks (§ Bolts, "The bullet's explosion"), its ribbon drawn from the gun to
  the tower and bending off it. Then, as #287 take07's breakpoint log has it, blade by blade
  (the bolt's hits, normals and new ways), the blade bounces off the tower's block (breakable 46,
  type h_e989e46e, the take's 7f1de400) along [-0.22, 0.19, 0.96] (the take: [-0.19, 0.22,
  0.96]), meets the flat wall up and to the left 5.4 m on at 0.2 s (breakable 281, the take's
  7f20ee00, h_0536a5bc 5.9 m on) and turns along [0.41, 0.17, 0.89] ([0.39, 0.20, 0.90]), then a
  third object 9.6 m on at 0.4 s (breakable 336; the take's 7f21ce00) along [0.99, -0.08, -0.13]
  ([0.98, -0.06, -0.17]), and flies on to its life's end. That second hit is take01's and take06's
  big flash up-left of the tower 0.1 s after the first (#287 take06 f12 and f15 at 30 fps; the
  demo's frames 98 and 101 of a capture at 30 fps). The demo's blades had flown 35-45 m to the
  ground: the recipe aimed at Brutus's facing, 11 degrees left of the take's blade, which met
  another facet of the tower. It leaves the cutter's scorch (h_1801e63d).
* A blade fired straight at a wall comes back past its owner and never hurts him (#287 takes
  10-13: Brutus's 105 hp all through, take13's second blade reaching the floor at his feet; the
  sweep's filter leaves the shooter out, FUN_00225440's +0x48). The demo's, at take13's spot
  (the map, character and weapon above, `BF_TEST_GOTO=-182.73,-47.2,-188.07,-46.9
  BF_TEST_GO_AT=100 BF_TEST_FREECAM="0.3,-182.73,45,-45.58,0,-89.9" BF_TEST_TELEPORT=0.5
  BF_TEST_FREECAM_OFF=0.7 BF_CAMERA_PITCH=-0.35 BF_TEST_INSTANT_KILL=0
  BF_TEST_TRIGGER=3.0-3.08,5.0-5.08,7.0-7.08 BF_SHOT_LOG=1 BF_COMBAT_LOG=1`, the tester's):
  each blade strikes the wall (breakable 281) 4.4 m out at 0.1 s, comes straight back down
  through where Brutus stands and meets the ground at his feet 5.8-6.6 m on at 0.2 s, with no
  hit on him. The recipe before (Brutus at -46.85, pitch -0.10, the first spot's free camera)
  sent its blades back past him, missing, so it didn't test the filter.
* Held fire one blade every 0.38-0.47 s (2.5 a second; take01: 26 -> 18 in 3.5 s).
* Reload (`--test BF_CHARACTER=0 BF_TEST_WEAPON=1545a4e4 BF_TEST_CLIP=3 BF_TEST_TRIGGER=2.5-7.0
  BF_AMMO_LOG=1`; the label also names the enemies' variant h_1bb3c9c7, so the hex): the clip
  3 -> 0 a blade every 0.4 s, the reserve drops 30 when the cooldown runs out 0.4 s after the
  last blade, and the clip reads 30 14 frames (0.93 s) later. take02's watch log: 3 -> 0 at
  2.735 / 3.136 / 3.537, the reserve 100 -> 70 at 3.939, the clip 30 at 4.944.
* Damage on a frozen Filthy Mutant 9 m out in front of the tower (take03's spot; m03_a has no
  Confed trooper type, take03's spawn: the map, character and weapon above,
  `BF_TEST_GOTO=-182.6,-43.5,-186.3,-51.7 BF_TEST_GO_AT=100
  BF_TEST_FREECAM="0.3,-182.6,45,-41.88,0,-89.9" BF_TEST_TELEPORT=0.5 BF_TEST_FREECAM_OFF=0.7
  BF_TEST_NPC="0.8,h_10a12c64,enemy,frozen,-186.3,-51.7" BF_TEST_NPC_HP=1.5,1,300
  BF_TEST_INSTANT_KILL=0 BF_CAMERA_PITCH=-0.27 BF_TEST_TRIGGER=3.0-3.08,4.5-4.58,6.0-6.08,7.5-7.58,9.0-11.0
  BF_COMBAT_LOG=1`): every blade lands on its chest (x1), 7.9-10.0 (the roll 5.625-7.875 x the
  enemy step 1.4), one every 0.38-0.44 s in held fire; take03's taps took 9.1, 10.2 and 9.8,
  its held fire 8-11 each every 0.4-0.5 s. The white star flashes on the body (take03 f10 of 3.15
  s on), then the flesh's cutter hit.
* Not checked again (as before #53, generic): the dry fire and its hint, the switch (take02), the
  pickup (take04).

**PU Cutter** (#65, Brutus on e40; nothing of its own to build: the bolt's flight (#136), the
bounces (#238) and the explosion (#53) are the Feral Cutter's, with its own data: 100 m/s, 80 m,
4 a second, 10.5-13.5, plut_cuttertrail_ribbon in flight and plut_cutter_hit where it strikes.
The bullet's `<mirv>` count 3 is dead data: its mode is 0, NO_MIRV, plan §2):
* take01's spot, the cliff 10 m out (`-- --test BF_MAP=e40 BF_CHARACTER=0 BF_TEST_WEAPON="PU Cutter"
  BF_TEST_GOTO=-30.73,151.62,-31.65,151.23 BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.06
  BF_TEST_TRIGGER=2.0-2.08,3.5-4.5`): an orange streak from the gun to the reticle, the white star
  at the cliff with spikes after it, the dark mark; held fire one every 0.27 s (3.75 a second;
  take01 ~3.8). The cliff (the terrain: no bounce) is 10.3 m (pitch -0.14) to 11.7 m (-0.06) from
  the muzzle there, just past a first sweep's 10 m at 100 m/s, so the star comes 0.2 s after the
  shot where take01's came at 0.1 s (the HUD's 49 -> 48 at f12-f13 of 5.20 s on, the star at
  f18-f19): the take's spot or aim is a little closer, not measured.
* Reload (`--test BF_CHARACTER=0 BF_TEST_WEAPON="PU Cutter" BF_TEST_CLIP=3 BF_TEST_TRIGGER=2.5-6.0
  BF_AMMO_LOG=1`): 3 -> 0 a blade every 0.27 s, the reserve 120 -> 70 0.26 s after the last, the
  clip 50 13 frames (0.87 s) later; take02's watch log: 2.742 / 2.942 / 3.243, the reserve at
  3.545 (+0.30), the clip at 4.447 (+0.90).
* Damage at 7 m (take03's spot: `BF_TEST_GOTO=-0.5,166.98,-7.0,164.4 BF_TEST_GO_AT=100
  BF_TEST_NPC="0.5,shrike soldier@e40,enemy,frozen,-7.0,164.4" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_INSTANT_KILL=0 BF_CAMERA_PITCH=-0.3 BF_TEST_TRIGGER=3.0-3.08,5.2-5.28,7.4-7.48,9.6-11.0
  BF_COMBAT_LOG=1`): the taps took 15.7, 17.5 and 16.1 (the roll x 1.4 on a x1 part), take03's
  16.0, 17.9 and 16.1; held fire 14.8-17.3 a blade, each 0.1 s after its shot. Aimed higher
  (`BF_CAMERA_PITCH=-0.2`) the taps strike the head (x5, 78-88) and the held fire's blades pass
  just over it to the standing stone 25 m out and bounce off it, as take03's held fire missed
  the enemy and its blades left the stone to the right (why the held fire's blades go higher
  isn't looked into).
* A blade on the body (#287): take03 shows the white star, then a green blotch on the chest with
  sparks for ~5 frames (f13-f18 of 3.0 s on at 30 fps). The demo washed the whole Shrike
  cyan-white for 6 frames: the two cutter lights, the explosion's (cutter_hit's light-effect)
  and the flesh hit's (wmat_fleshit_cutter's, made by hand in play_fx.rs, `CUTTER_LIGHT`). Both
  left out, the demo's hit is take03's blotch; the explosion's alone left out, the wash stayed:
  the flesh's light did it. It stood 0.3 m off the body (the demo's), where its effect type puts it 2.70 m back along
  the shot (its light-effect light_hit_cutter, alignment 0, its emitter's tz 2.70: `Glow::back`;
  the explosion's, played from the data, already stood there). There, the body shows the blotch
  and a faint light, as take03 (the capture's frames 98-105 of `BF_TEST_TRIGGER=3.0-3.08
  BF_CAPTURE_FPS=30`). The LZRs' flesh light (light_hit_laser) moved the same way, 2.67 m back.
* The bounces off e40's standing stone (take05, take03): § Ricochet and pass-through above. A 25
  m hit comes on the third sweep, 0.3 s after the shot (take07: ~0.3 s), at the same damage (a
  bolt has no falloff; take07 17-18 a hit): the sweep's rule, not run again here.
* Not checked again (generic): the dry fire, the switch (take02).

**Rail RVR** (#67, Hawk; nothing of its own beyond the beam's spawn below: the pass-through
(#238) and the beam (#237), from its data: an instant ray, 90 m, 2.5 a second, 13.25-18.25
BALLISTIC with no falloff, n1 1, n2 500 and the pass flag, no scope; railhit_s with
light_hit_rail where it strikes, rgun_perp_s, rgun_side_s and rgun_beam in flight; on e34 the
start's letterbox hides the HUD: add `BF_NO_SQUAD=1 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`
to the e34 recipes below for it, camera.md, the tester's):
* The beam (§ Tracers and beams) at e34's gate 10 m out (take01's spot, `-- --test BF_MAP=e34
  BF_CHARACTER=2 BF_TEST_WEAPON="Rail RVR;Foley 356 Tact" BF_TEST_GOTO=-52.76,31.02,-52.9,31.5
  BF_TEST_GO_AT=100 BF_CAMERA_PITCH=-0.08 BF_TEST_TRIGGER=2.0-2.08`; the Foley as the second gun
  so the one stowed on Hawk's back doesn't hide the view): the spark at the hit, red-violet arcs
  from it down toward the gun turning blue over ~0.15 s, then the hit's blue sparks, as take01
  f19-f41 of 3.05 s on. Its rings were all spawned within a few metres of the gun, a bright tube
  round the camera; they're now strewn along the ray as the game's cube spawn puts them
  (`Compiled::game_spawn`), but stay brighter and many more than take01's and take06's faint few
  (§ Tracers and beams, "Not matched").
* At e40's cliff 25 m out (take06's spot: `-- --test BF_MAP=e40 BF_CHARACTER=2 BF_TEST_WEAPON="Rail
  RVR;Foley 356 Tact" BF_TEST_GOTO=-16.2,152.68,-16.69,152.58 BF_TEST_GO_AT=100
  BF_CAMERA_PITCH=-0.05 BF_TEST_TRIGGER=2.0-2.08`): the arcs from the gun up to the reticle and
  the white-blue glow at the muzzle (take06 f10-f17 of 2.5 s on, at 30 fps); the ray goes into
  the cliff (the terrain, its one wall) and on to its 90 m, the beam's length, hidden behind it.
* Held fire one shot every 0.4 s (2.5 a second; take01: one per ~0.4 s).
* Reload (`--test BF_CHARACTER=2 BF_TEST_WEAPON="Rail RVR" BF_TEST_CLIP=3
  BF_TEST_TRIGGER=2.5-6.5 BF_AMMO_LOG=1`): 3 -> 0 a shot every 0.4 s, the reserve drops 24 0.47 s
  after the last and the clip reads 24 16 frames (1.07 s) later; take02's watch log: 2.757 /
  3.158 / 3.560, the reserve 200 -> 176 at 3.961 (+0.40), the clip 24 at 5.065 (+1.10).
* Damage at 7 m on e34's Shrike Soldier (take03's spot: `-- --test BF_MAP=e34 BF_CHARACTER=2
  BF_TEST_WEAPON="Rail RVR" BF_TEST_GOTO=-52.76,31.02,-54.78,37.72 BF_TEST_GO_AT=100
  BF_TEST_NPC="0.5,h_e96829a0,enemy,frozen,-54.78,37.72" BF_TEST_NPC_HP=1.0,1,300
  BF_TEST_INSTANT_KILL=0 BF_CAMERA_PITCH=-0.32 BF_TEST_TRIGGER=3.0-3.08,5.2-5.28,7.4-7.48,9.6-12.0
  BF_COMBAT_LOG=1`): with the crosshair on its legs, as take03's (its hits flash at the knees),
  every ray strikes a calf (x0.5) and goes on through into the ground: 9.9-11.5 a hit (the roll
  x 0.5 x 1.4), take03's nine 9.75-12.77. On the head (`BF_CAMERA_PITCH=-0.12`) 101-111 (x5).
  take03's run of misses in its held fire (11.8-13.5 s) isn't reproduced: the demo's stays on.
* Pass-through can't be seen in #67's takes (take01's wall has nothing visible behind it). #287's
  R1 and R2 takes settle it: § Ricochet and pass-through ("Against #287's takes": through every
  character, stopped at the far side of the first object it enters).
* Not checked again (generic): the dry fire and switch (take02), the pickups (take04, take05),
  the far enemies (take09, take10).

### The guns of weapons batch 2 without footage (#56, #64), and the Rail CLVR's (#66)

Nothing of their own to build: each fires through the shared machinery from its own data, and
was run on the test map (`-- --test BF_NO_SQUAD=1 BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=4.5,20,4.5,20
BF_TEST_NPC_HP=1.0,all,300 BF_SHOT_LOG=1 BF_COMBAT_LOG=1`, dm Shrike Soldiers frozen on the
line). The Rail CLVR is compared below with #287's takes 14-17 (`todo/287-weapon-shots/`), which
also hold the Ion Beamer's I1 shot (take22) and the MNR-7's (takes 18-21), filmed since and not
compared here. Without a take, what the others look like and their numbers against the game are
open; the shots that would settle them are listed with each (the draft's R1, R2 and I1 are in
todo/drafts/done/bolt-ricochet-passthrough.md).

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
1.4), never the one behind; held fire one bolt every 0.53 s (2 a second: 16 frames), clip 100.
"Penetrates armour" is its damage-type 6 (PARTICLE) against each target type's factor (generic,
`play_damage.rs`); "some types of walls" are the objects the code lets it through (a placed
object, a gate, a crate, a broken object's remains: the ray result's kind 2), never the terrain,
which ends it. Blocked on footage: shot I1 (a bolt through a crate or gate leaf into a frozen
enemy, and at a frozen enemy with another behind), the bolt's and its hit's look
(paralyzer_bul_master, paralyzer_hit_master with light_hit_energy), its rate, reload and damage
against the game, and which of a level's walls it goes through (a take at a level wall).

**MNR-7 Bio** (#64; h_e671e0f8, light: Hawk, `BF_CHARACTER=2 BF_TEST_WEAPON="MNR-7 Bio"
BF_CAMERA_PITCH=-0.16 BF_TEST_NPC="0.3,shrike soldier@dm,enemy,frozen,4.5,10"
BF_TEST_TRIGGER=2.0-2.08,3.5-6.0`): each trigger cycle fires three darts at once (its burst of
3, 0 s apart, walked by the burst's 3.5 degrees: #115's pellets; the log's three ways lie 3-4
degrees apart) for one round off the clip (50 -> 49 -> 48: FUN_0022fe00), one cycle every 0.53 s
(2 a second); each dart is a bolt at 80 m/s with its own roll (10-12 BIO) and its 40 m range (a
0.5 s life), and lands 0.2 s after the shot 9.6 m out (14.0-16.0 on a x1 part; 81.5 on the
head); one to three of them hit the Shrike, the walked ones passing it. The green darts
(bio_bullet_pistol and its trail) and the green bio flash where they strike (bio-flash_hit,
bio-shrap_hit, light_hit_bio). Blocked: the bioreactive kill (#147); on footage, the darts' walk
(a wall 10 m out with the camera still), the damage on a frozen enemy 7 m out with its HP
logged, the reload, and the darts' look against the game.

### The scope: levels, zoom, ramp and lockouts (#62, #61, #7)

`src/bin/play.rs` (`scope_levels`, `level_zoom`, `press_scope`, `scope_input`, `follow_camera`),
the skills' hooks in `src/bin/play_skills.rs` (`SkillKind::scope_levels`, `scope_zoom`,
`scope_view`), the zoom times in `bf/character.rs` (`ScopeOptions`). From default.xbe, as #62's
spec (`todo/62-l-shot-75/spec.md` §3) and the analyst's answers
(`todo/weapons-plan/answers-batch1.md` §10, §11) read it; the views are in
[squad.md](squad.md) ("Scopes").

* **Levels** (FUN_00124840): a gun whose zoom (h_efce7f77, data+0x16c) is above 0 has a scope
  (the MK-ASLT and the Kman Auto 2, the Foley 3, the L-Shot-50 5, the L-Shot-75 10); none with a
  gun in the second hand (Tex's berserk). How many levels is the holder's skill's +0x28 asked
  with 1: the base returns it (0x262a40), one level; Flint's (0x13c910) one more, two, with any
  gun that has a scope.
* **A level's zoom** (FUN_00124920): 1 at level 0, else the gun's zoom through the skill's
  +0x2c: the base gives it (0x222cf0); Flint's (0x13c930) gives z + max(10, z + 5) past her first
  level (10.0 at 0x13c94f, 5.0 at 0x3a4fd4): the L-Shot-75 10 then 25 (x2.5), the L-Shot-50 5
  then 15 (x3), the MK and the Kman 2 then 12, the Foley 3 then 13. The takes: x2.47-2.52 (#62
  takes 01, 06, 08, 10) and x2.97 (#61 take01). The demo's x2 second step (a guess) is gone.
* **The button** (CT_AIM_SNIPE, 0x122399-0x122479; right mouse): out of the scope, into level 1
  when the gun has levels and the button's lockout (char+0x7c8) is out, setting it to 0.85 s
  (0x12241a); below the top level, up one with no lockout test; at the top, out of it once the
  lockout is out (FUN_001249b0, which sets the lockout to 0.9 s, 0x124a1c). A reload
  (FUN_00120d40) leaves it the same way, as do the demo's switch, death and Tex's berserk. The
  in sound (the type's +0x260, the character's h_00dd1fe0) plays with each level gone up
  (FUN_00124340), the out sound (+0x264, h_02c94d34) as it leaves.
* **The ramp** (FUN_0011aa20, 0x11acc6-0x11ae79), linear in the zoom: in the scope the zoom
  (+0x7ac) climbs at (the level's zoom - the level below's) / T_in to the level's, never past it
  (FUN_0011ae80), and the progress (+0x7cc: the camera's move to the eye, the model hidden, the
  crosshair to the middle) at 1 / T_in; out of it the zoom falls at the top level's zoom / T_out
  (FUN_0011af10) and the progress at 1 / T_out. T_in and T_out are game-options'
  `<character-options><h_ebe93276 h_1858b846="0.3" h_03a75ca9="0.3">` (0x19f640 stores them at
  settings +0x4e4 / +0x4e8; FUN_0012dfa0 copies them to 0x4699d0 / 0x4699d4). FUN_0011aa20 runs
  twice a frame for the controlled character (its own tick FUN_0011caa0 at 0x11cf03, and its
  follow camera's FUN_000702a0 through FUN_0011c0b0), so a level takes 0.15 s and the lockouts
  0.425 and 0.45 s (`SCOPE_CALLS`; medium confidence on the double call). The takes: 0.17 +-
  0.03 s a level, the L-Shot-50's second level growing 2.0-2.3 zoom units a game frame against
  2.22. The demo's eased ramp (`SCOPE_RATE`, exponential, ~0.3 s) is gone. The game's
  FUN_0011af10 never meets a gun without a scope in hand: its switch swaps the gun at the clip's
  event, after the scope was left and its zoom fell (that gun's top of 0 would stop the fall). The
  demo's gun can change at once (a test hook, a squad hand-back) with the zoom still up: then it
  falls at its own zoom / T_out. A gun in the second hand, or none, keeps the game's top of 1.
* **The view** is the skill's (+0xa8: FUN_000b4070 shows the scope frame h_1807ec04 only while
  its bit 0 is clear, and 0xc6030 makes Flint's overlay only when it's set): Flint's (0x2d5e40)
  1, no frame, her overlay of a corner smear and a dotted ring; everyone else's 0 (Tex's 8:
  nothing reads bit 3), the frame. Both are drawn by `src/bin/play_scope_view.rs` (#283), one
  pass over the picture in the console's sRGB values: the frame 1:1 from the middle, darkening
  the picture to 40% outside its rim, whole at the press and growing 3x as it fades on the way
  out; the overlay added, easing in from 3x to 2x over 0.17 s ([squad.md](squad.md), "Two kinds
  of scope view", with how it was checked). `BF_SCOPE_VIEW_LOG=1` prints their state.
* **The field of view** at a zoom is FUN_0006c720's 2 atan(tan(FOV / 2) / zoom) from the game's
  follow camera's 70 degrees across its 4:3 picture, 55.4 down (`FOV_Y`, #269,
  [camera.md](camera.md)): the L-Shot-75's first level shows 8.01 degrees across, 6.01 down, its
  second 3.21 / 2.41 (take15 / 16's live zfov), the MK's 38.6 / 29.4 (#62 spec §3.3). Before #269
  the demo drew 45 degrees down, every view 1.27x the game's. `BF_SCOPE_LOG` prints each level's.
* **The camera in the scope** is the follow camera's own (todo/126 answers Q1): blended by the
  progress toward 0.5 m behind a point 0.5 m above the root node, the aim re-aimed so the
  crosshair's point stays under it (camera.md, "The scope"); the demo's eye at the head went.

Test hooks: `BF_TEST_SCOPE=<level>[,<from s>]` holds that level (at most the scope's top) from
that time on, under `BF_TEST_GOTO` too; `BF_TEST_SCOPE_PRESS=<s>[,<s>...]` presses the scope
button at those times, with its lockouts; without either, a hook's aim (`BF_TEST_AIM`,
`BF_TEST_FIRE`, `BF_TEST_AIM_HOSTILE`, brain.md) with a gun that has a scope is its first level, as
before; `BF_SCOPE_LOG=1` prints each press, each level change and the ramp's zoom and progress on
every frame they move.

Verified (`BF_SCOPE_LOG`, `BF_SOUND_LOG`; e40's cliff at take05's spot, `--test BF_MAP=e40
BF_TEST_GOTO=-17.21,152.02,-17.70,151.93 BF_CAMERA_PITCH=-0.10 BF_NO_SQUAD=1`, the scope on the
rock 24 m off, `BF_CAPTURE_FPS=30`; e34 opens letterboxed with no HUD):
* Flint (`BF_CHARACTER=1`) with the L-Shot-75, presses at 2, 3 and 4 s: level 1 climbs 1 -> 10 by
  2.0 a game frame (0.15 s), level 2 10 -> 25 by 3.33, out 25 -> 1 by 5.56 a frame (0.144 s).
  Level 2's picture against level 1's: x2.49 (frames 85 and 115, the best of the scales about
  the middle, r 0.96; the takes x2.47-2.52). The L-Shot-50: x2.99 (r 0.96; take01 x2.97), its
  second level 2.22 a game frame. Her MK-ASLT: levels 2 and 12.
* The lockouts (presses at 2.0, 3.0, 4.0, 4.2, 4.6, 4.8, 4.9, 5.2): the entry at 4.2, 0.2 s after
  leaving, is refused (0.5 s left); 4.6 enters; 4.8 goes up to level 2 (no test); the exit at 4.9
  is refused (0.25 s left); 5.2 leaves. Flint's in sound 1fd08ea5 with each level up, her out
  sound 03e0936d with each exit.
* Hawk (`BF_CHARACTER=2`) with the Kman Auto: one level, 2x, in the frame, the gun's crosshair in
  the middle; an exit 0.3 s after entering refused; the shared sounds 135ec2ce / f9ffc972.
* A reload in the scope (`BF_TEST_CLIP=1`, the L-Shot-50's last round): out of it as the reload
  starts, with the out sound, and in again 1.2 s later.
* `BF_TEST_FIRE=1` with Tex's MK on the test map: level 1 at 0.93 s, the scoped rounds 5-6 game
  frames apart (its scoped rate 6/s). `BF_TEST_SCOPE=2,2.0` with Flint's L-Shot-75: level 2 at
  2.07 s, zoom 25.

### Missiles: the Therm Sweeper (#73), the Sweeper V (#72) and the lobbed ones (#239)

`src/bin/play_missile.rs` (the camera shake in `src/bin/play_shake.rs`), the bullet's data in
`weapon::MeshBullet` (`src/bf/weapon.rs`), from default.xbe's projectile class
(0x226000-0x229000) and the weapon type's damage handler (0x22d0b0); every function cited here
is in [code-map.md](code-map.md). The measurements are the tester's specs,
`todo/73-therm-sweeper/spec.md`, from takes 01-07 there, and `todo/72-sweeper-v/spec.md`; #73's
takes 08-23 (a real reload, self-damage at a wall, homing on two enemies and an off-line one, the
open air, a side view, cover) came after its spec and are compared under **Verified**.

* **Which guns.** bullet-type 2 is BTYPE_MESH_PROJECTILE (the enum at 0x3bd63c): FUN_0022f1d0
  launches the bullet's mesh-name object (FUN_0022fb50), and every one flies here
  (`missile::flies`). Those steered at a steady speed (h_0b03ece5, flag 4) are the Therm
  Sweeper, which splits (a `<mirv>`), and since #72 the Sweeper V, which doesn't; no other weapon
  in any level's data has the flag. Since #239 the iKhan-GPL's round flies here too, lobbed
  (below). The seven bullet-type 2 weapons with a projectile object in the data are these three
  and the hounds' four Breath Attacks (h_0edc1617, h_1492c676, h_11582931, h_1c6108cb: lobbed,
  `<mirv>` count 1 mode 0, 24-45 m/s, blasts of 3.5-8 m); the demo leaves the creatures' own
  weapons (weapon-type 7, WTYPE_NATURAL, the table at 0x3bf714) on the plain shot, as e34's
  Shrike Hounds fire theirs at the squad and their breath has no footage (#83, #130).
* **Lobbed** (#239; `falls`, `GRAVITY`, `SPEED_DAMPING`): without h_0b03ece5, and with neither
  homing (flag 8) nor seeking (0x40), FUN_00226cc0 never steers the projectile (FUN_002266d0 is
  skipped, 0x226dd9-0x226dfc). Its body keeps the velocity FUN_00227ce0 gave it at the launch,
  the bullet's speed along the shot (vtable +0x58 adds the change through FUN_002b3b20: no mass
  term), and falls under the Ipion world's gravity, (0, -9.81, 0) (FUN_0016a250), its speed
  damped by 0.1 a second (the body template of FUN_00158cd0 / FUN_00158f80: 0.1 where Ipion's own
  template constructor, 0x2b4710, puts the speed damping's 0.01; #161's inference, by that
  layout), each game frame before its tick, as the physics step runs before the objects'
  (FUN_00169e20). **Which bodies fall** is the projectile object type's (`falls`): its `<base>`
  h_11e7d864 (base +0x35 -> the runtime type's +0xa8 bit 4, FUN_00154a20) has the root body's maker
  FUN_00158cd0 (game object vtable +0x158) take the body off the world's gravity controller
  (0x158ec0 -> FUN_0016a760: the core leaves the controller of priority 1000, vtable 0x38e7f4,
  0x2b4770, the one the Ipion environment makes beside its gravity at 0x2b49ff). It's set on
  exactly the two steady missiles' objects (h_e9327576, h_fa69b507) and on no other mesh
  projectile's (h_175a0adf, the breath objects h_0dc93533 and h_175340d6): the reviewer's lead,
  traced. The iKhan-GPL's (h_e9be70f0): 23 m/s, range 40, so a life of 1.74 s and a
  straight phase of 0.6 of it, 1.04 s; fired level from Brutus's muzzle, ~1.45 m up, it comes
  down 12 m out 0.53 s later. It goes off at its first contact (0x227b90: h_0bafbd6f set and
  n1 = n2 = 0; another projectile doesn't count), the world or a body, as any missile here: one
  blast. WM_MIRV_ON_APOGEE (its `<mirv>` mode 1) splits it instead on the tick its upward speed
  is under 0.5 m/s past its straight phase (0x2272b4-0x2272ea), when its life runs out, or at its
  proximity fuse (a hostile within its Damage radius 8 x 0.375 = 3 m that it has passed, looked
  for every 0.3 s), which sets its life to 0: 3 children h_175a0adf leave beside it, turned 14.4
  degrees (spread-max 0.16 x 90) at a random roll, **at the parent's speed then** (FUN_00227f80:
  without h_0b03ece5 a child's +0x208 is the length of the parent's velocity, 0x228748-0x228794;
  a steady parent's leave at the bullet's), and fall in their turn, NEUTRAL and without a target
  (no fuse), living 0.85-1.15 x 1.74 s, going off at their first contact or where they are when
  that runs out (h_0976a9dc). So a round fired at something short of ~12 m goes off on it as one
  blast; the bomblets come from the fuse when it passes an enemy, or from a shot more than ~10
  degrees up. Its node isn't turned (nothing steers it), so the drawn round keeps the turn it
  left with. The game runs a projectile's timers x5 while it's in a liquid (its +0x104, 0x226d22):
  not done.
* **An AI's lobbed rounds** (`missile::lob`, `missile::lob_holds`; **the demo's stand-in for the
  game's AI aim, untraced and not filmed**): FUN_0022e5b0 turns every gun straight onto its aim
  point, and fired so, level at a target 20 m off, an AI's round came down ~5 m in front of it and
  its own 8 m blasts killed it (e40's Shrike placement h_1f979a8a, which carries the iKhan-GPL).
  So an AI pitches a lobbed round up, the lower of the two pitches whose flight (the demo's
  gravity and damping at the round's speed, the split and the world left out) passes through its
  aim point, or, out of reach, the one that carries it farthest at that height; and it holds its
  fire while the aim point is within its own blast's reach (the Damage radius and
  `BLAST_PART_REACH` more, across). What the game's AI does about a lob isn't read.
* **The iKhan-GPL** (#55; h_e9be70f0, WTYPE_RIFLE, Brutus's in the takes): 0.66 rounds a second
  from a clip of 16 (ammo-type 9, for which no level defines an ammo item: no reserve, no
  reload), its fire sound f3ccb53d, its muzzle effect h_0d18cac3 (grenade_launcher_smoke and
  grenade_launcher_muzzle, no light; compiled with the missiles' rules, as every mesh projectile
  gun's launch is), and the bracket crosshair h_12a2c478. Its round is the projectile object
  h_175a0adf (model h_13a8dfb8, 0.04 m), the yellow glow with a ribbon behind it: the attached
  effect h_f788f9c4 (mirvgrenade_trail: mirvgrenade_ribbon and mirvgrenade_flare) and its sound
  10318b29, lobbed (above). **Its bomblets** are its own `<mirv>` children, three more
  h_175a0adf: the fuse makes them as it passes an enemy within 3 m, the apogee when it's fired
  more than ~10 degrees up; each goes off on its own. Each blast (a direct hit's, a timeout's or a
  bomblet's): the explosion type h_e26237da (exp_mirvgrenade: a white dome, sparks and a flash;
  light_explosion; the sound 065168c9), the bullet's impact sound e404fac6 (in no level's sound
  bank: silent, as the Sweeper V's), Damage 18-20 EXPLOSION (as written, min 20 and max 18) in a
  radius of 8 m, half a roll to every body it reaches (h_011cb154 false), the scorch h_e1e26e25
  (1.5 x 1.5, 25 s) on the ground under it, and the view's shake (18 x 0.00769 = 0.14 within 56
  m). A direct hit takes a roll first. **The other "iKhan-GPL"**: h_1f4a5a0a carries the gun's
  stringtable name and description but is the bioreactive death's vaporise blast (WTYPE_DATA,
  Damage 1.1-1.5 x the victim's max HP, BIOREACTIVE, radius 7: the invisible grenade h_0569a8b4
  that FUN_001252b0 leaves at the body goes off with it, #147), never held, placed or picked up.
  `BF_TEST_WEAPON="iKhan-GPL"` took whichever of the two the weapons' map gave first (the blast
  about half the runs: Brutus then held its 7 mm placeholder); a label two definitions share now
  takes the one that isn't WTYPE_DATA (`shots::test_weapon_key`).
* **The Sweeper V** (#72; `h_ed3f6b0c`, the same machinery with other numbers, spec §2): one
  rocket a trigger cycle, 0.5 a second (60 game frames), clip 8 of the squad's AMMO_10 rockets.
  Its missile object h_fa69b507 is the Therm's model h_07d84a52 with its own attached effect
  h_fb438237 (the same mtrail, mtrail#1, mtrail-flare and hiss 0021996c); its explosion type
  h_e2a10c7d holds the Therm's six effects, light and sound; its muzzle effect (h_002779c8, the
  launch dust and light_muzzle) and scorch (h_e6868a8d) are the Therm's. It leaves at 60 m/s
  along the shot after the accuracy turn (accuracy 90 throughout: +-0.33 degrees) and flies
  **straight**: with h_07cdf040 and h_f69b7d4c false nothing steers it to a target or an aim
  point, so FUN_002266d0 steers it along its own velocity (flag 4) and holds it at 60 m/s, and
  with `<mirv>` count and mode 0 it never splits. It goes off where it strikes the world (not the
  level's invisible blockers and hinters, which Ipion's pair table keeps off every body but a
  character's: #298), a body
  or an object (a body struck takes a roll of 27-37 first), at its proximity fuse (Damage radius 9
  x 0.375 = 3.375 m), or where it is when its life runs out (range / speed = 200 / 60 = 3.33 s;
  h_f0a56297 0, so no second timer; h_0976a9dc). Its blast is 9 m: half a roll flat to every
  body it reaches (h_011cb154 false), x0.7 on the shooter's side; its centre is the bullet's
  radius, 0.3 m, off what it struck. The shake is 37 x 0.00769 = 0.285 within 31.5 m, to nothing
  at 63 m. Its impact sound h_f55abaf3 is in no level's bank: it plays nothing (the game's does
  the same: it asks and finds nothing). The Shrike Heavies that carry it (e40's placement
  h_17bc6519, 8 rockets) launch the same rockets at the squad, with no fuse (the fuse is a human
  player's missile's, FUN_00112800).
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
  follows life / start (FUN_00228c10). No gravity: their objects' h_11e7d864 takes their bodies
  off it (**Lobbed** above), as the takes show (take20's children climbed straight at ~8.5 degrees
  for 168.6 m in 5.58 s). The steering wouldn't hold one up: the Sweeper V's, along its own
  velocity (flag 4 alone), takes nothing across it, and a missile under gravity would bend as
  freely as an unsteered round.
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
* **Proximity fuse** (h_0bafbd6f): for a player's missile (FUN_00112800, read as "a human
  player's": medium confidence) with a target or a side other than NEUTRAL
  (0x22731c-0x22732d), a hostile (only its target, if it has one) within Damage radius x 0.375
  (1.69 m for the Therm, 3.375 m for the Sweeper V) of it, to their root, that it has passed.
  Its clock (+0x218) gains each tick's time while it's under 0.25 s and looks (and is cleared) on
  the tick it's 0.25 or more (the C of FUN_00226cc0's fuse block): every 9th tick at 30 Hz, 0.3
  s, which for the Sweeper V is every 18 m of flight, so a rocket passing a body is seen only if
  a check falls while it's within 3.375 m past it. It runs after the give-up and the timers.
  The parent then splits; a child (only while it holds the split's target) goes off; a Sweeper
  V's rocket goes off.
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
  body struck takes a roll of Damage min..max (84-96, FUN_0022c4d0; min + U x (max - min) as
  written, FUN_0022c940: the iKhan-GPL's min 20 and max 18 roll 18-20), x the hit-location factor,
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
  body struck directly takes a roll, then half another. **What it reaches** (#72): the game tests
  no distance. The blast is a physics ball of the Damage radius (FUN_0022c2e0, made as its delay
  runs out) and FUN_00224a90 walks the bodies the engine lists as overlapping it: a character
  is gathered when one of its collision-bone bodies is among them, its ray going from the centre
  to that bone, one entry per character (the analyst, todo/weapons-plan/answers-batch1.md §9).
  The demo meets the ball with the hit parts' collision surfaces as drawn
  (`damage::part_distances`, the same surfaces the shots strike), a part counting within the
  radius and 0.62 m more (`BLAST_PART_REACH`), and the line of sight goes to that part's bone;
  lying knocked down, the ragdoll's bones (`lying_parts`); a body without parts, its standing
  cylinder 1.35 m round (`BLAST_BODY_REACH`). The engine's own overlap rule isn't in the game's
  code, so both margins are fits to three takes: Tex was hurt by the Sweeper V's blast at the gate
  10.2 m away six times in six for a 9 m radius (#72 take01), and by the Therm's at 5 m but not at
  6 m for 4.5 m (#73 takes 15, 13). With Tex aiming at the blast, his arms out, the demo puts his
  nearest part 9.50-9.57, ~4.2 and 5.18-5.20 m from those blasts (his feet axis 10.30, 4.94, 5.94):
  a margin in [0.57, 0.68) for the parts, [1.30, 1.44) for the cylinder, of which these are the
  middles. #73's 0.6 m cylinder, fitted to the Therm's takes alone, fell short at the Sweeper V's
  10 m. The analyst's live test (a frozen enemy facing the blast against side-on, at r + 0.8 and
  r + 1.2 m) would tell whether the arms decide. Loose pickups are thrown. The scenery takes the same
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

Test hooks: `BF_MISSILE_LOG=1` prints each missile's launch (a lobbed one's ends "lobbed (gravity
9.81, damping 0.1 a second)"), seeks, retargets, apogee ("at its apogee: rising -0.45 m/s, 5.09 m
above its launch"), split (children,
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
* The Sweeper V's takes (#72), with `BF_TEST_WEAPON="Sweeper V" BF_NO_SQUAD=1` and taps
  `BF_TEST_TRIGGER=3.0-3.15,5.7-5.85,...`: take05's cliff `BF_MAP=e40
  BF_TEST_GOTO=-17.21,152.02,-17.70,151.93 BF_CAMERA_PITCH=-0.10` (the goal 0.5 m ahead: he
  turns to it and stands; the take's view `-- --test BF_TEST_FREECAM=0.5,-14.48,28.9,152.66,77,-7.5`);
  take01's gate `BF_MAP=e34 BF_TEST_GOTO=-53.64,31.07,-53.76,31.56`, and 1.5 m back
  `-53.31,29.62,-53.43,30.11`; take03's enemy at the gate spot, `-- --test BF_TEST_INSTANT_KILL=0
  BF_TEST_NPC=0.5,h_1c5780f8,enemy,frozen,-55.61,37.79 BF_TEST_NPC_HP=1.0,1,300`; the open air
  `-- --test BF_TEST_GOTO=4.5,20,4.5,19.5 BF_CAMERA_PITCH=0.17` (to see it,
  `BF_TEST_FREECAM=0.5,4.5,1.3,23.5,0,14`); the fuse, the same spot with `BF_CAMERA_PITCH=-0.115
  BF_TEST_NPC=0.5,brutus,enemy,dummy,6.2,-15.5` (33 m out and 1.5 m off the line, where the
  second check, 36 m out, finds him just passed); an enemy's launches, e40's Shrike Heavy
  placement `BF_TEST_NPC=0.5,h_17bc6519,enemy,fight,2.15,168.43` with take06's spot.
* The iKhan-GPL's takes (#239, #55: `todo/55-ikhan-gpl`), on e40 with Brutus as in the takes,
  `BF_MAP=e40 BF_CHARACTER=0 BF_NO_SQUAD=1 BF_TEST_WEAPON=e9be70f0`, taps
  `BF_TEST_TRIGGER=3.0-3.08,5.2-5.28,7.4-7.48`: take01's cliff at 10 m
  `BF_TEST_GOTO=-30.63,151.66,-31.10,151.48 BF_CAMERA_AIM_AT=-40.22,28.5,147.40` (the crosshair
  1 m up the cliff from its foot, as the take's sits; at the foot itself the round comes down 2 m
  short on the rising ground); take08's at 25 m `BF_TEST_GOTO=-16.45,152.86,-16.94,152.76
  BF_CAMERA_AIM_AT=-40.6,28.0,147.9`; take03's frozen enemy 7 m out `-- --test
  BF_TEST_INSTANT_KILL=0 BF_TEST_NPC="0.5,shrike soldier,enemy,frozen,-6.62,164.35"
  BF_TEST_NPC_HP=1.0,1,300 BF_TEST_GOTO=-0.19,167.13,-0.65,166.93` with
  `BF_CAMERA_AIM_AT=-6.62,27.6,164.35` (on his chest) or `-10.01,27.6,162.12` (past his side, the
  fuse's case); the apogee, open air from take08's spot turned round,
  `BF_TEST_GOTO=-16.45,152.86,-15.96,152.96 BF_CAMERA_PITCH=0.35`.

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
* **The Sweeper V** (#72, the recipes above; `BF_MISSILE_LOG`):
  * take05's cliff (three taps): each rocket strikes it 22.4-22.6 m from the muzzle, 0.367 s
    after the launch (11 game frames; the take's fire-to-blast on the sound 0.356-0.413 s, mean
    0.378). From the take's view the glow sits over the reticle from the frame after the launch,
    the blast is at the cliff's foot on the reticle, and the scorch shows there from ~1 s, a
    little left of and below the take's. The blast is the Therm's, with its known excess (below).
  * Self-damage at the gate (take01's spot, three taps): Tex takes 7.4, 5.9 and 6.0 from blasts
    whose ball meets his nearest part 9.50-9.57 m away (the take: 6.5-7.3 per blast, six of six;
    the rule 0.21 x the roll, 5.7-7.8); from 1.5 m further back nothing. The Therm at the gate
    (the part reach, the same runs): 5 m from the panel 19.7, 17.9, 18.0 (takes 10-16: 18.6-20.1),
    6 m nothing (his nearest part 5.18-5.20 m away); take14's Flint 1 m to his left the same as him
    (19.7, 17.9, 18.0); take22's enemy behind the panel never reached (behind cover), take23's in
    front of it 65.5 and 59.6 (the take 65.9, 62.0, x1.4); take06's dummy one share per shot
    (65.2, 62.7; the take 61.6-64.6).
  * take03's enemy at 7 m (the e34 Shrike Heavy frozen, 300 HP, four taps): the direct hit at an x1
    bone, 46.9-49.3, then the share 0.07-0.25 s later, 19.7-25.0: 67-74 a rocket. take03 logged
    two half-roll drops (19-26 each, 41-47 a rocket): its Shrike was frozen 1.2 m off the floor (y
    30.24 against Tex's feet at 29.04), so its rockets met the legs (x0.5) where the demo's frozen
    NPC stands on the floor and is met at the hips. Every drop is inside the rule. Tex, 6.4 m
    from each blast, takes his own share (5.9-7.5); the take didn't log his health.
  * The open air: the rocket goes off where it is, 202.0 m out, 3.37 s after the launch (101 game
    frames: the 101st from the float sum of the 1/30 s steps; the code's 3.33 s and 200 m), a small
    orange burst with no white flash (LOD 0) and no shake (the camera beyond 63 m).
  * The fuse: three rockets past a dummy 33 m out and 1.5 m off the line each go off at their fuse's
    second check, 36.0 m and 0.600 s after the launch, 0.8-1.5 m from him; he takes a share each
    (19.5-24.5). Not filmed (the spec's shot list).
  * An enemy's (e40's Shrike Heavy placement, 15 m away, fighting): a rocket every 2.0 s at Tex's
    hips, each a direct hit (12.5-13.1: the roll x0.62, his type's multiplier, x0.6) and its share
    (8.5-10.7).
* **Lobbed** (#239, the iKhan-GPL's recipes above; `BF_MISSILE_LOG`):
  * The Therm Sweeper on #73's take01 setup and the Sweeper V on #72's take01 setup log the same
    missile lines as main's build, line for line (only the kits' load lines change order, as the
    weapons' map does run to run).
  * take01's cliff (three taps): each round leaves at 23 m/s and strikes the cliff's foot 9.5 m
    from the muzzle, 0.40 s after the launch, the blast out of Brutus's reach (his nearest part
    9.5 m away). The take, at 60 fps: the burst on the cliff at the crosshair's bottom bracket
    0.47-0.50 s after the launch glow's first frame, on two shots. Not matched: 0.07-0.1 s
    longer in the take, cause not found. The code's launch is the bullet's speed exactly. Both
    iKhan takes (01 and 08) would fit a stronger damping, ~0.7 a second; the reviewer noted that
    the objection to it assumed the steady bodies take the damping too. They do, by the code: the
    steady missiles' bodies come from the same template (FUN_00158cd0's 0.1; h_11e7d864 takes them
    off the gravity only), so at 0.7 the Sweeper V's steering would hold ~52 m/s where #72's
    take05 timed 59.5 on average. Kept at the code's 0.1; a follow-up. Where the take's crosshair
    sat on the cliff, and so how far its round went, is read from the picture.
  * take08's 25 m: each round comes down on the sand 12.4-12.7 m from the muzzle, 0.53 s after the
    launch (the take: ~0.47 s after the glow, "~8-10 m ahead" by eye).
  * take03's spot, a frozen Shrike Soldier 7 m out (the take's was a Shrike of e40's spawn
    template 834ae160), aimed at his chest: every round strikes him 0.23 s
    after the launch, a direct roll of 25.5-26.2 (18.2-18.7 x1.4) and then its blast's half share
    of 12.8-13.8, 0.07-0.25 s later; Brutus takes 3.8-4.2 of his own blast (5.7 m from him).
    Aimed past his side (the round 0.7 m from his hips), the fuse splits it 6.8 m out at 0.3 s
    and the three bomblets come down 7.7-13.8 m out, 0.33-0.6 s after the launch: two or three
    half shares of 12.7-13.9, 0.07-0.35 s apart, and no direct roll; Brutus takes one share when a
    bomblet lands within ~8 m of him. take03 logged both patterns: 26 at once (shots 1, 4 and 7),
    two or three shares of 13.1-14.0 0.1-0.2 s apart (shots 2, 3, 5), 26.2 then 13.2 (shot 6),
    300 down to 85.8 over seven rounds where the demo's seven direct hits take 273. That reads as
    most of the take's rounds passing him and splitting at the fuse (an inference from the shares'
    count and spacing; the take's blasts hide the rounds), which the demo's body, a cylinder 0.4 m
    round, doesn't let a round aimed at him do (the game strikes his collision parts; a round
    through the gap between his legs isn't checked). The rolls: 18-20 as the code has them (the
    shares 13.1-14.0 = x1.4 of 18.7-20.0), where the demo rolled 18 every time before.
  * The apogee, 20 degrees up in the open: the split 1.07 s after the launch (its straight phase
    out, rising at -0.45 m/s), 21.3 m out and 5.1 m up; the bomblets fan out (three trails in the
    capture) and land 32-39 m out, 1.7-2.1 s after the launch. Not filmed.
  * An AI's (the tester's repro, `-- --test BF_MAP=e40 BF_NO_SQUAD=1 BF_CHARACTER=0
    BF_TEST_GOTO=17.15,175.43,17.15,175.0 BF_TEST_NPC=0.5,h_1f979a8a,enemy,fight,17.7,196.0`):
    on #239's first build (the tester's run) every round of the Shrike's came down ~5.3 m in front
    of it and its own blasts (9.0-9.7 each) took it from 64 HP to 0 by 11.4 s, Brutus untouched.
    Now it pitches them 13.4 degrees up and each strikes Brutus 19.5 m off, 0.9 s after the
    launch, a round every ~1.5 s: 6.8-7.4 (the roll x0.62, his multiplier, x0.6) and the blast's
    share 5.5-5.9; it takes nothing itself, and Brutus is down at 14.9 s (main's plain shots hit
    him every 1.5 s too). With the squad (without `BF_NO_SQUAD`) Hawk and Tex shoot it from 37-40
    m, and it gets two rounds off at Brutus, both hits. Put 6 m from Brutus (17.7, 181.5) it holds
    its fire (no round in 10 s); at 10 m (17.7, 185.5) it fires, nearly level, and its own blast
    misses it by 0.4 m (its nearest part 8.97 m away). `cargo test` checks the lob's pitch (~11
    degrees for 20 m level) and that the flight passes within 2 cm of the target.
* **The iKhan-GPL** (#55, the recipes above with `BF_TEST_WEAPON="iKhan-GPL"`, which now gives the
  gun every run; `BF_CAPTURE_FPS=60` against the takes' 60 fps):
  * take01's look: the launch is an orange smoke beside the gun and a yellow glow leaving it with
    its ribbon, as the take's (its smoke shows on its second shot; the take's glow is a round halo
    ~90 px across for its first 0.1 s, the demo's a streak: the demo's camera sees the round's way
    from the side, where the take's is behind it, and the flare is a facing-3 quad stretched along
    its motion). The burst is a white dome at the crosshair on the cliff's foot; above luminance
    200 (in 640 x 480 units², 15 fps from the burst) the take's grows 0.6k, 3.2k, 5.2k, 6.7k and is
    gone after 0.33 s, the demo's 0.5k, 1.5k, 4.7k, 4.0k, 3.9k, 2.3k, 1.2k, over 0.47 s: the same
    light all told, 0.7x at its peak and longer. The scorch shows under the crosshair from ~0.7 s
    (the take ~0.8 s).
  * take02 (`BF_TEST_CLIP=3 BF_TEST_RESERVE=0`, the trigger held): three rounds 1.53 s apart (the
    take 1.51-1.60), no reload with no reserve, then a dry click (f740ecdd) on a tap and the
    switch.
  * take06's frozen enemy 25 m out (Brutus at the take's spot, 17.13, 175.49, the crosshair on the
    enemy): each round comes down 14.4-14.6 m out, 9.5 m short of him, and doesn't reach him (the
    take: no hits). take05's 45 m wall is the same arc: down on the sand ~12-15 m out.

Not matching or not done:
* The explosion is too big: 1.5x the take's bright area on take04's cliff (two fireballs) and
  1.6x on take06's sand (one), taller and whiter. Nothing in the game's explosion depends on
  the surface (the analyst's Q2: placement, frame, slots and LOD are the same on both), and no
  exp-rocket appearance uses shape 3 (Q4). Left for a ticket of its own; its leads: the angle
  fade of the facing-3 quads near edge-on (exp-rocket-add's, up to 13.6 m), which the demo
  doesn't do, and the added sprites summed in linear light (#104). A third, now done (#269): the
  demo's view, free camera included, was 45 degrees down where the game's 4:3 picture is 70
  degrees across, 55.4 down, so the same scene was drawn tan(27.7) / tan(22.5) = 1.27x larger
  across the 640 x 480 units, 1.61x in area, about the excess measured (1.5x, 1.6x): expected now
  0.93x and 1.0x, to be measured again ([camera.md](camera.md)). take20's open-air flame is
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
* The demo's follow camera put Tex over the crosshair, so the launch and the blasts were mostly
  behind him (before #126: its 45-degree view and the aiming dolly; the game's camera now,
  [camera.md](camera.md), not checked against these takes again). On e40 Tex and the standing
  stones draw black (main too): not this ticket's.
* The other effects keep the demo's quads, perp rules and count, and have no LOD: the shape 2
  appearances elsewhere (half the game's size), facing 3 on non-perp appearances (the Psionic
  Blast's muzzle ball), the Frag's facing 3 quads. A gun's hit lights (an effect type's light
  on a bullet's hit) are played since #57 (**Bolts and their hits**).
* The Sweeper V (#72): no real reload was filmed (the takes' reserve edit; #73's takes 08 / 09
  stand in: Tex's heavy reload clip); nor its fuse, its end in open air or a side view of a launch
  (the spec's shot list). The blast's margin past its radius is a fit (`BLAST_PART_REACH`: the
  engine's overlap rule isn't in the game's code). The shove of a frozen enemy with no damage after
  a rocket (take08 / 09, 3-9.5 m) isn't matched: the blast never pushes a character (FUN_00224a90
  pushes loose objects only, and deals what it pushes), so it's something else of the physics
  (the analyst, answers-batch1 §13). Its 0.3 m ball strikes a body where the demo's ray meets the
  cylinder: a pass within 0.3 m of the body misses here (the hit part's back-off, `STRUCK_BACK`, is
  #73's 0.5 m).
* Lobbed (#239): take01's flight is 0.07-0.1 s shorter than the take's (above, cause not found);
  a round aimed at a body always strikes the demo's cylinder, where most of take03's passed and
  split at the fuse; the creatures' Breath Attacks keep the plain shot (#83, #130); a
  projectile's timers running x5 in a liquid aren't done.
* The iKhan-GPL (#55): its explosion's flames. The take's rise and spread up and out from the
  dome (+0.13-0.33 s); the demo's lie flat along the ground. exp_mirvgrenade has a radial field,
  exp-mirvgren_rad.fld (class h_03d20af6; its parameters 5.9, a curve 1 to 0.001, a curve of 370
  falling to 62 over 0.2 s and 0.43, read by Freelancer's FxRadialField as its radius,
  attenuation, magnitude and approach), which `src/ale_fx.rs` doesn't run, and two air fields
  (exp_mirvgrenade_air.fld, a wind of 0.004 m/s at approach 1.49; exp-mirvgren-air.fld, 1 m/s at
  0.3) left off (`AIR_FIELDS`): run without the push, the first would stop every particle where
  it's born. The weapon panel reads "16 / 0" where take01's reads "16": the game's panel
  (0xcbdd0) writes "%d / %d" (0x40d654) only when the held gun's data +0x174 is set and it doesn't
  recharge, "%d Regen" (0x40d664) for a recharging one, else the clip alone (FUN_000bb920); data
  +0x174 names an object type that FUN_0022b6d0 makes with each weapon (0x22d820) and a
  character's animation event makes again (0x118390), which reads as the gun's magazine. What sets
  it isn't found, so the demo keeps "clip / reserve" (a code question). No level defines an ammo
  item for the iKhan-GPL's ammo-type 9.
* The iKhan-GPL (#55): the round's glow and ribbon are pale cream, ~(229-240, 200-215, 125-136)
  (the tester's), where the takes' are a saturated gold, ~(245, 172, 67). The ribbon's colour,
  mirvgrenade_ribbon.app's (0.808, 0.545, 0.149), and the flare's, (0.788, 0.710, 0.271) to (0.792,
  0.459, ...), are added as linear amounts, as `src/ale_fx.rs` adds every additive appearance's
  (the convention play_fx.rs gives: the console adds to the picture's stored gamma values, and
  amounts taken as linear come out about as bright as its sums over a mid-grey picture); encoded,
  the ribbon's is (232, 194, 108). Taken as the console's gamma values it's (206, 139, 38), which
  over the take's dark cliff (~40, 30, 30) adds up to about the take's gold. Not this effect's slip
  but the blend every added effect is drawn with (and checked against its takes with): a
  follow-up, e.g. adding the effects in gamma values.

Sounds (`src/bf/audio.rs`) come straight from the game's banks: `sounds-<level>.xmb` maps sound
ids to Xbox ADPCM data in `sounds-<level>.mem`, decoded to 22 kHz WAV in memory. Each character's
archetype names its jump grunt and footstep type (Tex 1, Flint 2, Brutus 3, Hawk 4), and
`world-materials.xmb` gives every surface four footstep variants per type plus landing and slide
sounds. Footsteps fire when a toe comes down into the lowest quarter of its swing in the clip
being played; in a liquid the step is the liquid's wading step or nothing, by the depth
(FUN_00119020's liquid branch, #5: levels.md, "What a liquid plays"). The liquids' sounds are
requests with play parameters of their own (`WorldSound`: a position heard by the library's
distances from the listener and dropped beyond its max distance, a volume, a pitch in octaves
played as the speed); `BF_SOUND_LOG` prints each with its kind, volume, pitch and distance. Surface sounds live in level archives: `BF_LEVEL=<name>` (default `e01`) picks the
level; `mp1`'s bank is merged in too, since a mission only carries its own squad's footsteps.
On a level, its sound-triggers play their own voices (#223: voice lines, door and gate sounds,
ambience, the music cues; 3D ones by DirectSound's distance law from the camera): see
[triggers.md](triggers.md), "Sound- and dialog-triggers", and their `BF_SOUND_LOG` lines
("trigger voice ...").

Test hooks: `BF_AUTOPILOT=1` plays a scripted run (idle, run, turn, sprint, aimed walk,
backpedal, dodge, standing jump, running jump, aimed fire, weapon switch, firing on the run); `BF_TEST_EXPLOSION=1` sets off the selected grenade type 6 m ahead every 1.5 s; with `BF_TEST_GOTO`, `BF_TEST_THROW=<s>[,<hold s>,...]` holds the grenade button (G) at those times (see **Grenades** for the rest of its hooks); `BF_ANIM_PROBE=<script>,...` prints each character's clips by script name (duration, root motion over a cycle, the clip's own speed h_f9628571, events); `BF_DUMP_TEXTURE=<file>:<hex id>` writes a decoded texture (raw RGBA after a u32 width and height). `BF_AUTOPILOT_SPIN=<rad/s>` makes it look down and turn the camera during the standing fire; with `BF_CAPTURE=<dir>` it saves every frame at a
fixed 15 fps (`BF_CAPTURE_EVERY=<n>`: only every n-th, for a long run at that step whose logs
are what's wanted, #294). `BF_MUTE=1` silences audio, `BF_SOUND_LOG=1` prints every sound event with its time
and action, `BF_DUMP_SOUND_IDS=<dir>:<id,id..|all>` decodes sound ids to WAV, `BF_DUMP_SOUNDS=<dir>` writes the character's jump and surface sounds as WAV files
and exits. `BF_DUMP_WEAPONS=1` lists every character's weapons as loaded (definition, model parts,
hardpoints, muzzles, fire sounds), then the map's definitions with more than one muzzle, and exits; `BF_DUMP_TYPES=1` (or `all`: every archive) lists the
character types, their motion sets' clips and the placements, and exits
([character-types.md](character-types.md)); `BF_SHOT_LOG=1` prints each shot's barrel and shot
directions; `BF_FEET_LOG=1` prints, every frame, the controlled character's soles (the posed
model's lowest skinned vertex) and its two foot bones over its ground, whether it moves, and the
overlays playing with their times (a pickup, a use, a reload, a throw: #293's check of the
pickup clips' feet). Camera hooks ([camera.md](camera.md)): `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` set the
start (with `BF_TEST_GOTO`, `BF_CAMERA_PITCH` holds the pitch throughout, e.g. to fire down at
something; held to +-60 degrees), `BF_CAMERA_DISTANCE` sets every character's boom,
`BF_CAMERA_AIM_AT=x,y,z` turns the aim so the crosshair's ray passes through a point (the recipes
here from before #126 aimed with `BF_CAMERA_PITCH` on the old camera: the same pitch now puts the
crosshair elsewhere), `BF_CAMERA_LOG=1` prints the camera, `BF_VIEW_YAW` turns only the rendered
view about the pivot (aim unchanged), `BF_START_WEAPON=1` starts with
the second weapon. Captures wait 60 frames for shaders to compile before the clock starts.

### Damage: the difficulty step and the hit parts (#127)

`src/bin/play_damage.rs`, from default.xbe's damage path as the analyst read it
(`todo/damage-factor/answer.md`; every function cited is in [code-map.md](code-map.md)). A hit
on a character goes:

* **The roll** (FUN_0022c940): Damage min + U x (max - min), x a character shooter's skill's
  factor (its +0xec at 0x22c9a3; `Hit::skill`, the log's "x the shooter's skill"): Flint's x2 on
  15% of her rolls while her targeting is on and x1.5 scoped (0x13cc70, [skills.md](skills.md#flints-advanced-targeting-124));
  1 for the others; then x each of its modifiers' (char+0x55c, `Hit::power`, the log's "x the
  shooter's power-up"): BRUTE POWER's x2 for its 45 s ([pickups.md](pickups.md), "Power-ups",
  #15). Every roll a shooter makes takes them: a shot's on a body or a breakable object, a bolt's
  and a ray's, a missile's direct hit and its blast's own roll. The Hat's char+0x754 (x4,
  FUN_001478d0) isn't in.
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
  0.85 while on, `Hit::victim_skill`, skills.md); then each of its modifiers' (`Hit::victim_power`,
  "x the victim's power-up": FORCE POWER's x0.5 on any hit but a BLADED one, #15); then
  FUN_002232d0's factor for the damage type. Every hit on a character
  reaches it: the guns' hits and blasts, the scenery's damage areas (FUN_0021ade0), falls
  (FUN_0012b740, DTYPE_NORMAL), FUN_00223160 (Brutus's charge), the damage plate's frames
  (FUN_001f8510, #228: [triggers.md](triggers.md), "The damage plate"). Only the trainer's `!hit` skips
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
