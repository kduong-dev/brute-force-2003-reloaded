*Part of [pickups/](README.md), the former page `pickups.md`.*

## Weapon pickups (#17)

`play_weapon_pickups.rs`, written from default.xbe's code (the addresses below; the reading is
`todo/17-weapon-pickups/leads.md`'s) and checked against take11 of `todo/28-tutorial` (the
tutorial's L-Shot-50), `todo/61-l-shot-50` take04 (Tex), `todo/52-confed-lzr-50` take04 (Brutus)
and `todo/reference/DNA + Weapon Pickups.mp4`.

* **What lies.** A gun is never taken by walking over it; X takes it. Lying guns are:
  * a level's weapon-object placements (object type 20 WEAPON, reticule-action 2 RACT_PICKUP),
    drawn as their own model in the placement's pose, no glow and no idle effect, with a full
    clip: the tutorial's L-Shot-50 by the crate at (-109.9, 17.3, -39.1) (placement h_f6a8a9b5),
    e34's iKhan-GPL at (98.6, 41.0, -49.4), the campaign's others (leads §1). A spawn-trigger's
    template isn't one: every multiplayer map's weapons, e34's Therm Sweeper, m02_a's second
    MK-ASLT and m02_c's iKhan-GPL lie as its copies, on their side under the trigger, from its
    spawns (#278: "Pickups from spawn-triggers");
  * the gun a swap drops, and a dead character's guns (the ragdoll's start, FUN_00127290, drops
    whatever both slots hold, FUN_0014dfd0; release_weapon's dead path, FUN_00117660 at
    0x118a64, does the same where a clip carries the event), with the rounds they had; they fall
    and tumble as the loose pickups do ("Pickups are loose" above).
  * a creature's weapons too, as the code has them: m04_a places Psionic Blast x2 and Breath
    Attack (class 7, ammo-count -1, the 7 mm dummy model h_ff18f12d) at (-207, 44.5, 76.8), and a
    dead Shrike Hound drops its Breath Attack. Nothing sets them apart: a placement whose
    h_1cb7d5f2 is false becomes a pickup whatever its class or model (0x14a0f7 -> FUN_0014a170),
    the dead's slot guns drop whatever they are (FUN_0014dfd0), the object update hands every
    weapon out of a holder to the use candidates (FUN_00157c40 -> FUN_000ba1c0), and neither the
    candidate loop (FUN_000bd180) nor the tracker (FUN_000bf1e0, FUN_000bf2f0) looks at a class,
    a model or the rounds. The squad's holders refuse class 7, so walking there shows "Tex cannot
    pick up Breath Attack." and the red ring over next to nothing. Not filmed (a shot at m04_a's
    spot and over a dead hound would settle it); where the game rests a model-less one (its
    collision-type is 0) isn't known: the demo puts it on the ground as every pickup, the
    placement has it 2 m up.
  * Taken, a gun is gone for good: nothing puts a placed campaign weapon back (leads §4.6), nor
    a medkit or a box (#18). A spawn-trigger's copy comes back as its trigger's next spawn
    (#278).
* **The vanish timers** (#278; the gun's +0x1a8, FUN_00157c40, `Vanish`): while one runs it counts
  down, and at 0 the gun is taken out of the world (the tick loop FUN_000d9840 -> FUN_000db3e0).
  Outside a mission (the multiplayer maps: the session's level kind isn't 0x17506391):
  * a gun a swap drops, 20 s (release_weapon, 0x1189e5);
  * a dead character's guns, 20 s (FUN_0014dfd0);
  * a gun a walk-over takes rounds from, 10 s, whether one ran or not (FUN_0011c170, 0x11c3fd).
  In a mission only a dead squad member's guns get one, 300 s (FUN_0014dfd0); the enemies' and a
  swap's dropped guns stay, as every campaign take has them. A use's start on the gun stops its
  timer (FUN_00121670). A level's placed gun and a spawn-trigger's copy have none.
* **The target** (the HUD's use tracker FUN_000bf2f0 and its caller 0xbf110): of the lying guns
  at most 1.7 m above or below the controlled character's position (the hips: its node) and 2.5 m
  away across (the target it has at 0.7 of its distance, so kept out to 3.6 m), in the front half
  of the aim (the dot of the aim's heading with the way to it at least 0; a target that isn't a
  pickup needs 0.5) and in its line of sight, the one most in front. None while scoped, dead or
  in a cutscene. A gate panel in reach keeps the prompt unless the gun is more in front.
* **The prompt** (play_hints.rs `draw_prompt`, FUN_000aa600's weapon case) at the hint line's
  place, as the panel's:
  * the character can take it when one of its two slots takes the gun's class (FUN_000aad20 ->
    FUN_000a9910 -> the slot holders' 0x14baa0; the classes are the character type's
    `limit-weapon`: Brutus 1 8 / 2 1, Flint 1 8 / 0 8, Hawk 0 8 / 1 8, Tex 2 1 / 2 1, with 0
    pistol, 1 rifle, 2 cannon, 8 none). Then "Hold [X] to swap L-Shot-50." (h_f1285f72, the gun's
    display name h_1dd3a49d), or for a gun with no magazine prop (h_0bfde1e8 "") and no ammo-regen
    "Hold [X] to swap iKhan-GPL. (16 shots)" (h_e456c40f, its rounds);
  * otherwise "Brutus cannot pick up Jax-iP." (h_0ff85d29), the words alone from the line's start.
  * So Brutus and Tex refuse the pistols (Foley, Jax-iP, LZR-10, Kman, MNR-7, Saryl-23), Hawk the
    cannons.
* **The ring** round the target (the ring widget, its update 0xa9bf0 and the reticle
  controller's 0xa9510; play_hud.rs): 48 units across once in, zooming in from 2.4 times that,
  fading in (its openness c up 1.2 a second, drawn 48 x (1 + 1.5 (1 - c^4)) at alpha c^4), and out
  the same way when there's no target; a new target takes it at once if it was in. Blue (23 53 d4)
  on one the character can use, red (a0 20 20) on one it can't, a lighter blue (3c 69 dc) while
  it's being picked up. The gate panels' ring is the same widget: 48 units, where it was 60.
  * It sits on the point the tracker keeps for its target (its +0x1c from FUN_000bf2f0):
    FUN_0015a480's, the target model's hardpoint h_10f81d34 if it has one (a gate panel's or a
    console's button: play_usables.rs `BUTTON`), else the target's position. None of the 59 weapon types' models or
    their parts has that hardpoint, so on a gun it's the gun's position, the middle of its model
    (the L-Shot-50's bounding centre h_e99750a9 is 1 cm from it). The reach and the line of sight
    are measured from FUN_0015a500's point, h_ee389721 or the position: the same for a gun.
  * It's projected through the view the frame draws (the camera's Transform as the cameras have
    just set it), not the GlobalTransform, which is the last frame's until PostUpdate: the ring
    was a frame behind its target whenever the view moved (#17's review). The camera shake
    (play_shake.rs) is added after and isn't followed.
* **The hold**: X (E) held 0.3 s on the target (the player controller's +0x50, 0x1332a0 ->
  FUN_000a9710), once per press. On a gun the character can't take the press only grumbles: its
  chatter line f2ce8278 through the chatter gate (its block: 100%, a 10 s replay delay: the 10
  this page read as a one-in-ten chance before #155; [chatter.md](../chatter.md)). **(#208)** While
  the character's skill says it isn't free to act (FUN_00117210 through its +0x14: Flint's targeting
  and the Wrath of Garner's skill, `skills::lets_act`) no hold arms (`use_target`'s `down`), nor on a
  console or a panel (play_usables.rs `activate`): **the demo's simplification**. The game doesn't
  gate the hold: the use update FUN_00120290 asks FUN_00117210 at 0x1207b9 only after the hold has
  completed, and a refused use stays pending while char+0x620 is above 0, else it is cancelled
  (0x12094c-0x12098c): it defers or cancels the completed use. Not filmed (#200's shot S1).
* **The use** (FUN_00121670, then the use update FUN_00120290):
  * the slot it goes to by the type's pickup mode (`limit-weapon` h_fb08d4c6, FUN_00121ac0):
    Flint and Hawk (1) the first slot that takes the class; Brutus (2) a cannon to slot 2, a
    rifle to the slot in hand; Tex (3) the slot in hand;
  * **the demo's Edward Kingman class (demo-own, #410, the product owner's rule;
    `slot_in_hand`)**: **the slot of the pistol in his hand** (mid-switch, the one the switch
    draws), so the gun he holds is the one let go and replaced, with no switch first. His data's
    `<limit-weapon one="0 8" two="8 8" h_fb08d4c6="0">` (objecttypes-m03_c.xml) is mode 0, every
    pickup to slot 1: with his second Kman Auto in hand (the demo's own extra, in a slot the data
    lets take nothing, #407) the use switched to slot 1 first and replaced the pistol on his hip.
    Only the type with the demo's holsters (play_holster.rs `Holsters`, the Berserker on a model
    with no stow hardpoints) has it; the squad's slots are the game's. In berserk the slot in hand
    is the right hand's gun; the clip waits there for the berserk's end, as Tex's does (the
    skill's overlay filter), and for his hands to be back from the holster (play_reach.rs), then
    plays as anywhere: his `Sc_w1_drop_pickup_w1` (ms_kingman's PICKUP_W1, the motion type every
    pickup of mode 0 asks for), which was already his pickup's clip before #410;
  * that slot switched into the hand first when it isn't there (the normal switch clip, both
    panels: #52 take04's Brutus over the LZR-50), then the drop-and-pickup overlay from the motion
    set (FUN_00121920's motion type: Tex's `Sc_w_drop_<hand>_pickup_<picked>` at its speed 1.1,
    Flint's and Hawk's `Sc_w1_drop_pickup_w1` / `Sc_w2_drop_pickup_w2` by the slot (Flint's at
    1.2), Brutus's `Sc_w1_drop_pickup_w1` for either slot), once no other overlay plays; a use
    whose clip can't start gives up after 5 s;
  * the clip is laid over the whole body, its root bone too (#293). Unlike the upper-body
    overlays (a reload or a use_item animates 36-44 bones and no root), the squad's pickup clips
    animate 51-68 bones, the legs and the root among them: it drops 0.42 m
    (Brutus), 0.50 (Hawk), 0.59 (Tex) and 0.64 (Flint) as the character squats, and moves up to
    0.3 m across over one planted foot (Tex's left foot steps out 0.38 m and back, Flint's steps
    back). The demo had laid every bone but the root, as for the other overlays: the legs folded
    under a body left at standing height, the soles up to 0.64 m off the ground (play.rs
    `apply_overlay`'s `root`). The takes bend the whole body down to the gun with the feet on the
    ground (#61 take04 6.6-7.1 s, #52 take04, take11 300.6-301.1 s);
  * while it plays the character is held where it stands: the stick neither moves nor turns it
    (FUN_001195c0 skips both while the playing overlay has h_f6a410e6, which every motion set's
    pickup overlays have: `Overlay::holds`, `GunUse::holds_still`). So the clip always plays
    standing, on every bone; before, a move during it laid the clip over the upper body only;
  * the gun is lowered, and there's no firing, aiming, reloading, switching, throwing or item use
    until the clip ends.
* **The clip's events** (FUN_00117660):
  * release_weapon: the gun in hand's rounds go into the squad's reserve of its ammo type as far
    as the stack takes them (FUN_0022b580, FUN_0014ce90) and it drops where the hand holds it,
    lying with the rest (take11's MK-ASLT, its 48 into the High ROF stack; on the test map,
    where the stacks start full, the gun keeps them). The weapon panel is empty until the grab
    and the crosshair hidden (take11 300.8-301.1 s); the switch's first sound (ff820fda);
  * grab_weapon: PICKUP_OBJECT from the lying gun to the level's triggers (the tutorial's router
    h_fa7e22af: "Click Right thumbstick to toggle sniper mode."), a new gun of its type in the
    slot and the hand with the lying gun's rounds, loaded at once from the reserve (FUN_0022daf0
    with 0), its name over its count on the HUD (play.md "The weapon's name after a pickup"), the
    switch's second sound (e1e97460), and the lying gun gone. The two sounds are the ones the
    takes have at the release and the grab (spectrograms: #52 take04 7.52 / 7.95 s, #61 take04
    6.57 / 6.88 s, take11 300.73 s). The new gun is held in the stance of its class for Brutus
    and Tex, of its slot for Flint and Hawk (play.md's table, FUN_0011aa20), and stowed where
    its slot's holder hangs guns: for Tex his left ramp, whichever slot (#297: a rifle Tex took
    in place of his Minigun had been held and stowed as the Minigun).
  * The prompt for the next gun is back from the grab, while the clip plays on.
* **Walking over a lying gun** (FUN_0011c540 -> FUN_0011c170), any living squad member, the AI's
  too: within 1.8 m below and 1.0 m above its position and under 1.34 m across, a gun with rounds
  whose ammo type a squad member's gun uses gives them to the reserve, keeping what doesn't fit,
  with the ammo item's pickup sound, PICKUP_OBJECT from it and the line "2x Sonic Ammo"; with the
  stack full nothing goes and the line reads "Sonic Ammo (full)" in red (a0 20 20), every frame
  it's walked over, on the HUD of the player who controls the walker alone (an AI squadmate on
  a full stack shows nothing: FUN_0011c6a0 with AL clear, #18) (the DNA video's "Bio Ammo
  (full)" and "2x Sonic Ammo"). In #61 and #52's takes nobody alive used the dropped guns'
  ammo: no line, as in the demo.
  * The guns that count are what each member's slots hold now (FUN_00122ed0 through
    FUN_0014dc50, which gives the hand holder for the slot in hand): between a release and its
    grab that slot holds nothing (`HeldWeapon::empty`), so the gun just dropped doesn't count by
    itself (no "(full)" line over it every frame with its stack full).
  * It runs before the clips' events: on a grab's frame the lying gun is still there for it, but
    the new gun's ammo type wasn't counted yet; a gun a pickup has grabbed is gone for it (#17's
    review: the other way round, the grab's own gun was walked over the frame it was taken, its
    rounds into the reserve a second time, "6 / 16" for "6 / 10").
* **The AI never swaps guns**: no goal of the game's picks one up (leads §9).
* **Not done, or not the game's yet:**
  * where a dropped gun lands: it falls from the hand with a small spin (the demo's; the game's
    drop, FUN_0014a1d0's physics, isn't read: shot list C);
  * the vanish timers aren't filmed (shot list C2): they're the code's (above);
  * the use's walk to the gun (FUN_00120290's phase 1, when the gun is beyond 2-3 m of a slot in
    hand or 1 m otherwise): the demo uses it from where the character stands;
  * take11's view turns about 20 degrees during the use (300.6-301.2 s, no stick input): not
    traced, not done;
  * the faint grey shape over the weapon panel between the release and the grab;
  * the game's slot left empty by a use cut short after the release: the demo takes the gun at
    once instead; a character with fewer than two guns doesn't pick into the empty slot;
  * BEFORE_USE_OBJECT (18) at the use's phase 2 isn't sent (nothing the levels have listens);
  * the times against the takes (the tester's measures, #17's review): the demo's release and
    grab come 0.1-0.2 s early, and Brutus's switch before his pickup (#52 take04) lasts about
    0.2-0.3 s longer than the take's. The clip rates are the data's (`speed`; the type's rate and
    the skill's factor, FUN_00150290's, are taken as 1), the hold the code's 0.3 s;
  * the shot list's open questions (leads §0): a part-used lying clip (the code reloads it at
    once at the grab), the hold time's edge, a swap while scoped;
  * a pickup kneeling: the game plays the clip's `_cr` version there (FUN_0014fd50 takes the
    crouch's suffixes in motion states 6 and 7), which all four have (Tex's
    `Sc_w_drop_rifle_pickup_rifle_cr` 1.17 s, Brutus's `Sc_w1_drop_pickup_w1_cr` 1.00 s, Flint's
    and Hawk's `Sc_w1_drop_pickup_w1_cr` / `Sc_w2_drop_pickup_w2_cr` 3.0-3.17 s, all whole-body);
    the demo plays the standing clip over the kneel's upper body, as its other overlays (not
    filmed).
* **Test hooks:** `BF_TEST_GUN=<weapon>[,<ahead m>[,<right m>[,<rounds>]]][;...]` lays those
  guns round the controlled character's start, 3.5 m ahead by default (the takes' carrier was put
  there and killed), full unless given rounds; `BF_TEST_SWAP=<s>[,<s>...]` holds X 0.5 s from
  each time (`BF_TEST_USE=<s>` holds it 1 s); `BF_WEAPON_LOG=1` prints the lying guns, the
  target as it changes (its class, the slots' classes and the mode), each use, its clip, the
  release and the grab with the rounds and the reserve, the walk-overs and the dead's drops (with
  their vanish timers), each gun that vanishes, and the spawn-triggers' gun copies, and (play.md)
  where each gun hangs whenever that changes, with its class and stance;
  `BF_FEET_LOG=1` (play.md) prints the soles' height every frame with the clip's time.
  A swap needs the gun to stay Tex's target through the hold, and `BF_TEST_GOTO` at a run
  overshoots a goal near the gun and loses it (at about 1.5 s), so a recipe walks
  (`BF_TEST_WALK=1`, which stops about 0.6 m short of its goal) or stands (`BF_TEST_GO_AT=999`).
  The smallest one: `--test BF_TEST_GOTO=0,0,0,-1.8 BF_TEST_WALK=1 BF_TEST_GUN="L-Shot-50,3.0,0.0"
  BF_TEST_SWAP=5 BF_WEAPON_LOG=1` (the gun is the target from 1.47 s, the pickup starts at
  5.33 s, the release at 5.67 s, the grab at 6.00 s).
* **Verified:**
  * the tutorial (`BF_MAP=tutorial BF_TEST_GOTO=-106.5,-43.5,-109.2,-39.8 BF_TEST_WALK=1
    BF_TEST_SWAP=4.0 BF_WEAPON_LOG=1 BF_HINT_LOG=1`, 105 frames; the walk stops about 0.6 m
    short of its goal, so the goal lies that far past where Tex should stand, and without
    `BF_TEST_WALK` he runs past the gun and loses it at 1.7 s, so the swap never starts) against
    take11 299.5-301.4 s:
    "Hold [X] to swap L-Shot-50." where the take has it, the 48-unit blue ring on the gun, zooming
    in as the take's (291.8-292.4 s); X held 0.3 s, `Sc_w_drop_rifle_pickup_rifle` at x1.1, the
    lighter ring and no prompt; the MK-ASLT released at 0.25 s (its 48 rounds to the High ROF
    stack, 500 -> 548), the panel empty and the crosshair gone; the grab at 0.55 s: "L-Shot-50
    6 / 0" name over count, the hint "Click Right thumbstick to toggle sniper mode." (blinking),
    "Hold [X] to swap MK-ASLT." 40 lower under it, the ring on the MK-ASLT at Tex's feet (this
    run: the target at 3.07 s, the pickup at 4.33 s, the release at 4.67 s, the grab and the
    hint at 5.00 s);
  * #61 take04 on the test map (`--test BF_TEST_GOTO=0,0,0,-3.0 BF_TEST_WALK=1
    BF_TEST_GUN="L-Shot-50,3.6,0.3;Jax-iP,3.3,-0.35" BF_TEST_WEAPON="MK-ASLT;Confed LZR-50"
    BF_TEST_SWAP=5.9`): "Tex cannot pick up Jax-iP." walking in, then "Hold [X] to swap
    L-Shot-50."; X at 5.9 s, the grab at 6.87 (the take's lying gun gone between 6.75 and
    6.85), "L-Shot-50 6 / 50", then the Jax-iP's red ring and refusal, as the take;
  * #52 take04 (`--test BF_CHARACTER=0 BF_TEST_GOTO=0,0,0,-3.0 BF_TEST_WALK=1
    BF_TEST_GUN="Confed LZR-50,3.5,0.25;MK-ASLT,3.3,-0.4" BF_TEST_WEAPON="Feral Cutter;Bower 20"
    BF_TEST_SWAP=5.9`): the switch to slot 2 (both
    panels), `Sc_w1_drop_pickup_w1`, the Bower 20 dropped, the grab at 7.87 s (the take's
    7.85-7.95), "Confed LZR-50 99 Regen" over "Feral Cutter", "Hold [X] to swap Bower 20.";
  * Flint over an LZR-10 (`BF_CHARACTER=1`, L-Shot-50 + Jax-iP): the switch to slot 2, then
    `Sc_w2_drop_pickup_w2` at x1.2, the LZR-10 in place of the Jax-iP, the L-Shot-50 kept;
  * the DNA video's lines (`BF_CHARACTER=0 BF_TEST_WEAPON="Jax-iR;Feral Cutter"
    BF_TEST_GUN="Jax-iP,3.0,-0.3,2;A50 Bioreactive,3.6,0.4" BF_TEST_RESERVE=50`): "Brutus cannot
    pick up Jax-iP." with the red ring and "2x Sonic Ammo" (24 / 52); the iKhan-GPL's "(16 shots)";
  * e34's placed iKhan-GPL is found and offered; a squadmate killed (`BF_TEST_KILL=1`) lets go
    of both guns with their rounds;
  * no regressions: the tutorial gate's panel (hints.md's command): its prompt, its ring (now
    48 units, zooming in), its use and the gate.
  * after the review: #61's recreation with `BF_TEST_RESERVE=10` reads "L-Shot-50 6 / 10" from
    the grab (Tex walks over the gun on the way in and takes its 6 into the 11mm stack, 10 -> 16,
    "6x 11mm Ammo", since a gun in the test map's squad uses that ammo; the grab's reload takes
    them back); the tutorial run as above, the same times and counts; m04_a (`BF_MAP=m04_a
    BF_TEST_GOTO=-203.5,78.5,-205.4,77.6`): "Tex cannot pick up Breath Attack." and the red ring
    on the creature weapons, as the code reads.
  * #293, the whole body: the #61 and #52 recreations above and Flint's and Hawk's
    (`BF_CHARACTER=1 BF_TEST_GUN="Confed LZR-10,3.5,0.25" BF_TEST_WEAPON="L-Shot-50;Jax-iP"`,
    `BF_CHARACTER=2 BF_TEST_GUN="Saryl-23,3.5,0.25" BF_TEST_WEAPON="Jax-iP;MK-ASLT"`, each with
    `BF_TEST_GOTO=0,0,0,-3.0 BF_TEST_WALK=1 BF_TEST_SWAP=5.9 BF_FEET_LOG=1 BF_VIEW_YAW=1.4`,
    125-135 frames):
    each squats to the gun with its feet on the ground, on every frame of the clip the soles
    within 3 cm of it (Tex -0.027 to +0.001 m, Flint -0.026 to +0.002, Hawk -0.016 to 0, Brutus
    0, his hand dipping 6 cm under the floor's plane at the grab), where Tex's had been up to
    0.58 m over it; the switch before Brutus's and Flint's pickup as before. With the stick
    pushed from 6.4 s (`BF_TEST_GOTO=0,0,0,-8 BF_TEST_GO_AT=6.4 BF_TEST_GUN="L-Shot-50,1.6,0.3"`)
    Tex stays where he stands through the clip (6.27-7.33 s) and sets off after it.
  * #299, a swap moves rounds and makes none: the release keeps in the gun what the stack doesn't
    take (0x118a09-0x118a22: FUN_0014ce90 returns how many went in, the gun's +0x1f8 less them)
    and the grab takes the lying clip and what it lacks from the reserve (`release_rounds`,
    `grab_rounds`, the unit test `the_swap_moves_rounds`). Eight swaps back and forth with
    `BF_TEST_SWAP=3.0,6.0,...,24.0 BF_WEAPON_LOG=1`, the clip in hand, the lying gun's and the
    reserve adding up the same after each: the tutorial's L-Shot-50 and MK-ASLT (the command
    above: 548 of High ROF, 6 of 11mm); the test map's laid L-Shot-50 (`--test
    BF_TEST_GOTO=0,0,0,-0.5 BF_TEST_WEAPON="MK-ASLT;Confed LZR-50" BF_TEST_GUN="L-Shot-50,1.6,0.3"
    BF_TEST_RESERVE=10`: 58 and 16), and two MK-ASLTs with shots between (`BF_TEST_GUN="MK-ASLT,
    1.6,0.3,20" BF_TEST_RESERVE=30 BF_TEST_TRIGGER=1.0-1.6,7.5-8.2,13.5-14.0`: 94, less each
    burst); e34's placed iKhan-GPL against Tex's Jax-iC (`BF_MAP=e34
    BF_TEST_GOTO=98.6,-47.0,98.6,-48.6`: the Jax-iC keeps its 36 with the Sonic stack full, the
    iKhan its 16, its ammo-type 9 having no ammo item); a dead squad member's guns
    (`--test BF_TEST_GOTO=0,0,0,0 BF_TEST_KILL=1 BF_TEST_SWAP=4.0,7.0,...`: Flint's L-Shot-50,
    walked over first, its 6 into the reserve and back at the grab); sdm_e34's Saryl-45 copy
    (the gun recipe in triggers.md, swaps every 3 s: 30 and 50 throughout, then the copy's
    respawn at 22.85 s lays a new full one, as it should). The test map's rack, the copies'
    respawns and the LZRs' recharge are the sources of new rounds.
