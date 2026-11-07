*Part of [squad/](README.md), the former page `squad.md`.*

* Hits come from the game's effect library (common/effects-common.ale, read with
  `tools/ale_tool.py`) and the flesh world material (21, WMAT_FLESH_HUMAN). Every hit plays its
  debris bundle: blood_puff (smokecard.tga mist), bloodsplat_s (diffuse_big.tga droplets in a
  jet, under a weak gravity field) and giblet (a blood.tga spatter). Each ammo type adds its own
  hit effect and sound: ballistic sparks (Flint, Hawk), the cutter's long white streaks with a
  pale blue flash light (Brutus) and the laser's orange embers with an orange flash (Tex). The
  emitter / appearance numbers in `play_fx.rs` are those nodes' parameters, read as Freelancer's
  ALE ones (Brute Force hashes their names). Each hit also leaves the character's hit decal on
  the ground past them (h_14410af1: the game's white splat textures tinted 128,34,34,220; about
  1 m across, as in the captures) for 40 s, fading over the last 20. Flint bleeds grey: she's
  synthetic flesh (world material 24, WMAT_FLESH_SYNTHETIC), which the data says through her
  parts' collision: the material most of her parts' triangles carry is 24, the other three's
  21 (#128: `Game::flesh`, read at spawn into `Loaded::flesh`; this was an inference from her
  grey decals before), whose debris bundle h_1f7ca034
  is blood_puff_b, bloodsplat_b and giblet_b - the same nodes in grey and blue-grey, slower
  and smaller - and whose hit sounds for the heavier ammo are metallic. Her hit decal
  (h_02d017b7) is tinted 50,50,50. Each hit and death splat is one of the type's hit decal list
  (h_fb4bfc68), the pool its second list (h_efc2a874); the squad's lists hold one decal each.
  (The Shrikes, flesh 23, lay theirs by the game's own rule and blend: [enemies.md](../enemies.md),
  "Blood"; the squad keeps this reading until #150.)
  Verified with `BF_CHARACTER=1 BF_TEST_SUICIDE=1 -- --test`
  captures against Brutus (`BF_CHARACTER=0`): grey mist and splats for her, red for him; and
  (#128) Brutus killing her on the test map, frame for frame as before the flesh came from the
  data (character-types.md, "Verified"). A
  hard hit (15+ damage: L-Shot, Bower, a close grenade) can knock a character down: the body
  goes limp (the game's ragdoll, [ragdoll.md](../ragdoll.md), thrown by the hit's push, harder at
  the top so it topples: the demo's start) for 0.7 s, thuds on the ground
  (the surface's landing sound) and gets back up over 0.45 s where it lies - the game uses
  Ipion physics for this (its constraint messages are in default.xbe, with MT_KNOCKDOWN).
* Death: the death cry (chatter e1213623 - the captures' deaths match its lines; ed35ffae for a
  fall's), a thud, a
  limp body that stays on a blood pool (the character's other decal, h_16d327fd), blood
  splashed round where they fall (the hit's blood effects sprayed up, and five of their hit
  splats on the ground within 1.4 m: the demo's choice, the game's own death blood isn't
  known; Flint's decals are dark grey, 50,50,50, in the data: she's a synthetic), their memory
  chip beside where they fell (below). Every line goes through the chatter gate and its one
  queue ([chatter.md](../chatter.md): the hurt lines, the death cry, the killer's line). The death
  cry's response picks the nearest teammate for its "X is down" reply ("Brutus is down!",
  "They've taken out Hawk."), whose variants need the alert value 0 where every squad member's
  is 10: so, as the code reads, no reply is heard (the demo said one half the time before #155;
  the xemu take A3 settles it). On the radar the portrait becomes the member's own skull (the characters'
  h_00c51907 icon) and the member's tab and health bar go.
