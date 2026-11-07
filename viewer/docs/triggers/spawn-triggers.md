*Part of [triggers/](README.md), the former page `triggers.md`.*

## Spawn-triggers

* **The template.** A spawn-trigger's `h_1299668b` names a placement: a character, an effect
  object, a pickup. As the trigger links (0x209210 -> FUN_000dfe90) the placement is taken out
  of the world and counts how many triggers spawn it (-1 for a re-armable one: `h_0f95f576`). Each
  spawn (FUN_000dfd90) clones it, and the last one of a counted template takes the template
  itself: that's how the tutorial's Tex (only trigger_spawn8 spawns him) seems to "move". So
  every placed character a spawn-trigger names (the tutorial's eight Red Hand, Tex and the five
  marines) is absent until spawned, and its collision isn't in the demo's arena either (arena.rs:
  Tex's template put a surface 1.7 m above the ground where his spawn stands him). Nor is a
  template drawn where it's placed (#230, `level_scene::spawn_level`, the same `spawn_templates`
  set: the tutorial's h_e5ab44a9, a pillar of type h_f02c006a at (50.3, 25.6, 80.6) 15 m from the
  start, stood there as a walk-through ghost; the game shows it only once the campaign is done).
  Nor a pickup: every pickup of the multiplayer maps is a template (sdm_e34's 54, its four
  power-ups among them), as are the campaign's supply drops and caches (e34's 9 spawns of 6,
  m02_a's 30 of 15 ...): their copies come from their triggers (#278, "In the world" below). A
  pickup template's model is kept hidden (`level_scene::SpawnTemplate`) to make the copies from.
* **Spawning.** OPERATE while armed spawns at once, or after `h_eab6952f` s (e40's delayed
  spawns); a trigger is armed once by its constructor (0x1fd680-0x1fd6e7), and SPAWN_RESET arms a
  re-armable one again (the tutorial's lava bursts: TIMER_PULSE -> SPAWN_RESET, then OPERATE).
  Re-armable doesn't mean unarmed: the constructor arms every one ((+0xe4 & 0xe8) | 0x28 at
  0x1fd6ba) and the parser only adds bit 2 for `h_0f95f576`.
  The copy goes to the trigger's own transform (FUN_00209620), named `h_14f15a34` ("tex_1",
  "TUTORIAL_terrorist_5": what the scripts and later triggers call it; a re-armable trigger's
  copies get no name, the runtime makes one up). It sends SPAWNED to its listeners, and the
  trigger OPERATED to its targets with the copy as the subject (FUN_002099e0). The triggers that
  list the spawn-trigger listen to its copies (FUN_001fe8a0, FUN_002096e0): router99's twelve,
  counting the base's kills.
* **On the ground.** A character copy stands on the ground under the transform, at once: the
  place call (vtable +0x50, 0x1159c0) runs the position setter 0x1154b0, which casts a ray
  straight down from the point (FUN_0012dc00 -> FUN_000dddd0, up to 50 m) and puts the feet on
  what it meets. take03's watch log has Tex at y 25.25 within 69 ms of his spawn at 28.53 (the
  terrain there: 25.25 in the demo too), no fall. The demo takes the highest floor below the
  point within 50 m (`on_ground`), for the copies and for the squad's start: e40's squad at 26.2,
  10.9 m below its triggers (Brutus's live read, xemu.md); m09_a's two Shrikes 6.7 and 9.5 m
  below theirs; the tutorial's sentry TUTORIAL_terrorist_5 on the wall walkway (27.25), 3.1 m
  below his. An effect stays at the transform (a game object's place, 0x159b70, casts no ray).
* **The squad member check** (FUN_00209900, #221). A copy that's a character of a squad type (its
  type's `h_e78efbb2`: the four, the tutorial's Confed marines) in a mission is checked against
  the session's squad. A clone (a re-armable trigger's, or one of a template several triggers
  name: FUN_000dfd90's count, `Triggers::takes_clone`) is a new character, whose set-up has set
  its type's member flag first (FUN_0012f440). From a trigger with `h_0fa4f510` (+0xe4 bit 5,
  set on nearly every one) it's kept if the session counts it in the squad (vtable +0xb8,
  0x83ee0: its type's save-game slot `campaign/save-game/character-N/h_03761692` true, or a
  player controlling its type, or no slot of its type), else taken away at once: its death
  (+0x144(1), which clears its type's member flag, FUN_00112830), out of the world, and no
  SPAWNED goes out (OPERATED still does). From one without, it sets its type's member flag
  (vtable +0xbc, 0x814b0) and is kept. So the tutorial's three start marines (Gonzales, Cogburn,
  Stout: re-armable spawn-triggers h_0d7313c9, h_167a4273, h_1f7d72e5, `h_0fa4f510` true), whose
  save-game slots aren't alive, never come with Tex's farcast and leave their member flags 0
  (take03-09's `1|0|0|0`); the V-squad's spawn64 and spawn76 (`h_0fa4f510` false) do come, in
  take11, and set theirs (`1|0|1|1`). The slots, the member flags and who else writes them are
  [campaign.md](../campaign.md)'s ("The squad's flags"). The roster check before it (0x8d430, the
  world's +0xbec) is taken to pass every squad type: an inference.
* **Which way it faces.** Down the trigger transform's -z, as the level's cameras look. Checked
  on Tex: after the opening (take03 at 32 s) the game's follow camera looks past him west, at the
  dark peak and the palms; the demo shows that facing -z, and the rock face east of the start
  facing +z (the start points' way in arena.rs). `h_131de144`, a world point the copy's AI is
  given (FUN_00017b60), isn't where Tex faces (wpoint194 is south of him); what the AI does with
  it isn't traced.
* **`h_eeae96e7` is the spawn fade-in** (s), not a "mode": FUN_00209620 writes it to the copy's
  char+0x73c with the fade state +0x734 = 0 and alpha +0x738 = 0, and FUN_0011bd00 raises the
  alpha by dt / that each frame. 2 s for Tex and the marines (the farcast), 0 for the Red Hand
  (at once). The demo doesn't draw the fade: a copy appears at once (logged).
* **In the world** (`spawn_copy`): a character template becomes a character of its type with the
  placement's weapons, personality and team (as `BF_TEST_NPC` spawns a placement, #129), with a
  brain from the trigger (its home-point, h_131de144 and `locomotion`: the walk home, the patrol,
  sight and the fight, #227, [brain.md](../brain/README.md)); an effect
  object plays its effect type's ALE effects once at the transform (the lava bursts, the farcast
  flashes; the farcast-out pad, which the game keeps until its destroyer, plays its emitters once
  here), with the game's particle rules (`load_recorded_facing`: its shapes, its count once a
  game frame, its facing; #228, checked against take08's lava bursts, which the demo's half-size
  quads drew as a string of small blobs); a pickup (an inventory-object or weapon-object, #278)
  becomes a copy of its template's model at the transform, named as the copy
  (play_pickup_spawns.rs): one whose type has no collision body (`collision-type` CT_NONE: the
  power-ups) stays where the transform puts it, with its idle effect (sdm_e34's STAMINA POWER at
  (-84.07, 29.78, 60.58), #35 take20's live object); another item lies on the ground under it as
  the level's own pickups do; a weapon lies on its side on the ground under it, a lying gun the
  prompt offers (#129 take38: mp1's walkway guns flat under their spawn glows). The item's place
  call is the game object's 0x159b70 (no ray); where its body comes to rest is the demo's. Taking
  a copy removes it and sends PICKUP_OBJECT from it; what comes back is the trigger's next spawn
  ("Subject relays").
* **The squad's start (#102).** A squad member (Brutus, Flint, Hawk, Tex) that a spawn-trigger
  spawns starts on the ground under that trigger's transform, named as its copy (`SquadSpawns`,
  play.rs `player_at_start`): the tutorial's Tex at (65.55, 25.25, 74.03), where take03 has him
  after the farcast; e40's Brutus at (-14.0, 26.2, 160.7) (#102's live reading). A member placed
  in the world instead (m09_a's four, whose warp-triggers move them: #222) starts at its
  placement. It carries its template's inventory, as the copy is the placement
  (`SquadSpawn::dress`): its `<weapon>`s (the tutorial's Tex two MK-ASLTs, h_0c3db625, 48 a clip,
  the icon take03 shows) and its items, pooled with the other members' into the squad's one store
  (play_inventory.rs, #281: [play.md](../play/items.md) "Items: the squad's starting inventory"): its ammo
  is the squad's reserve of that type (play_ammo.rs: his 500 of type 2, take03's 48 / 500), its
  grenades, medkits and other items the squad's. The level's other squad members aren't there: the
  tutorial has Tex alone (the marines are its own spawns). On a map without either (the
  deathmatch maps) the start points and the formation are as before. When the member's
  spawn-trigger fires later (the opening script at 17.09 s, #222) it's moved there; if the member
  isn't in the world to move (the map editor's stand-in leads), nothing is made. `BF_TEST_GOTO`
  still puts the leader where it says.
* **A member a spawn-trigger spawns is out of the world until then** (#222). The game spawns Tex
  with the opening cutscene's farcast at 17.09 s (take03's watch log: the template at its
  placement until then, clk 17102). The demo keeps him out (`Player::absent`: not drawn, not
  seen by the detectors, nobody's target, no script or warp binds him) until `spawn_copy` brings
  him in, so the start area's proximity detectors h_ea082132 and h_e30f11a4 send ENTER_DETECTOR
  at 17.27 s, as he lands. Under `BF_TEST_GOTO` or `BF_SCRIPT` the level's start scripts don't
  play and the squad is in the world from t 0, as before ([cutscenes.md](../cutscenes.md)).
