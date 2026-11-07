*Part of [pickups/](README.md), the former page `pickups.md`.*

## DNA canisters (#228)

* An item of group-type 9 (IG_DNA_CANNISTER, the IG_ name table 0x3be594): the tutorial's Confed
  Marine DNA h_f4933c12 (stringtable-name "Alien Technology Acquired!", display name
  h_1dd3a49d "Confed Marine DNA unlocked!") in the base building at (-62.5, 15.6, 18.8). A squad
  member within the walk-over's reach (above, 1.34 m across) takes it. #228 fitted the reach to
  take12 at 1.3 m, where the line came between Tex's feet at 1.41 m (295.30 s, no line at
  295.25) and 1.25 m (295.44 s, the line fading in at 295.5); the code's 1.34 is between them
  (#18). Its pickup sound plays, the line shows its display name, and it's gone for good (an
  unlock is once).
* Its PICKUP_OBJECT starts the tutorial's timer467 (4 s), whose pulse plays Bratton's
  dxud_ai_00060 (4.22 s) ([triggers.md](../triggers/object-events.md), "Object events").
* The end screen names it (`DnaFound`, play_endmenu.rs): the squad's inventory holds the
  canister in the game, and the end screen's DNA line (FUN_00108360) asks 0x83110 for the DNA
  found: the save's campaign/save-game/dna-archetype, else the first inventory item whose
  `<campaign-gameflag>` path (the canister's h_1d604469/characters/h_048b8580) is a game-options
  character's `gameflag-path` (h_f5a42d61's entries, FUN_00198b60), that entry's name (h_07f2f670:
  h_0ce3dafe "Confed Marine") then " DNA"; "No DNA Found" without one. The demo keeps the DNA
  canisters taken on the level and matches them the same way (`Game::option_characters`); the
  save's flag isn't kept (#27).
* What it unlocks (its gameflag-database 1: the global flag h_1d604469/characters/h_048b8580, the
  deathmatch's Confed Marine) isn't kept: the demo doesn't load the global flags.
* Verified (`BF_MAP=tutorial BF_TRIGGER_LOG=1 BF_PICKUP_LOG=1 BF_TEST_GOTO=-58.9,21.26,-62.22,19.23
  "BF_TEST_PANEL=a,1;a,1;a,1;a,1"`, 120 frames at 15 fps): taken at 1.60 s (1.80 s before #331), PICKUP_OBJECT ->
  timer467 -> sound463 at 5.80 s; "Confed Marine DNA unlocked!" under Tex from frame 27 to 87
  (4 s), as take12's line (its place on the screen is the pickup lines' layout since #314). With the objectives and the ending fired after
  it (`BF_MISSION_LOG=1 "BF_TRIGGER_FIRE=h_f0cac43d,10,2.5;h_065b02c7,10,2.6;h_065b02c7,10,2.7;h_065b02c7,10,2.8;h_f82e0ba6,ENTER_DETECTOR,4"`,
  380 frames): MISSION SUCCESSFUL at 14.07 s, "DNA Sequences Recovered" / "Confed Marine DNA"
  with the DNA icon at take12's place (frame 379 against take12 at 363 s).
