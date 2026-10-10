# The original game in xemu: trainer, takes, debug features (issue #103)

Reference footage of the original game is recorded in xemu by the `xemu` agent. Its tools live
outside the repo, in `D:\Emulators\Xbox\agent\scripts\` (playbook: `D:\Emulators\Xbox\agent\README.md`,
section "Trainer and takes"); what they rely on in `default.xbe` is summarised here. "Live"
means checked in xemu 0.8.122 through QEMU's gdb stub (e40 and m09_a, Brutus, 2026-10-09);
"static" means read from the decompiled code only.

* **`trainer.py`** (the VM is paused for each command). Characters have vtable 0x39ddd8
  (constructor FUN_00110d30) and sit in contiguous memory (virtual = 0x80000000 + physical):
  * the controlled character is found through the player controller object (vtable 0x39db74,
    per-frame update FUN_001332a0; +0x10 the character, +0x20 the pad port), by scanning RAM.
    The squad is the `std::list` whose head is `[[0x469ab0] + 0x1348]`: each node's +8 is an
    entry (vtable 0x39f878) and the entry's +0xe0 is the character (FUN_001351c0). Live.
  * health +0x54 and max health +0x50 (the combat target at +0x48; FUN_002232d0 subtracts
    damage, FUN_0010f830 clamps). Live: Brutus 105, Flint 90, Hawk 65, Tex 115.
  * position +0x458 (feet) and +0x464, velocity +0x470, yaw +0x434 in radians (bearing
    atan2(dz, dx) = 90° − yaw). Writing the position fields alone moves the body but not the
    model; teleporting calls the game's own setter FUN_001154b0 (vtable +0x38) on the game
    thread through the gdb stub. Live.
  * +0x830 the AI brain (vtable 0x3a4f04). The AI update FUN_0001afe0 runs the goal machine
    FUN_000377b0 only while the brain's mode (+0x58) is 0-5; mode 6 leaves the character
    standing, not shooting. That is the trainer's "freeze". Live.
  * items: `[[char + 0x2e8] + 0x1b4]` is the item store, with a vector of item pointers per
    category at +0xc + 0x10 × category; an item's count is the uint16 at +0x1a; the selected
    (category, index) is at `[char + 0x2e8] + 0x1c0`. Live: the Frag count followed writes and
    throws in the HUD.
  * +0x1d4 is 0 for the squad and 1 for enemies (e40; what it means is a guess).
  * `[0x469980]` is not the list of placed objects: its maps (FUN_00160da0) are keyed by type
    hashes (objecttypes), not by the placements' `name=`. `[0x469948] + 0x22c + 4n` are the
    player slots, whose +0xc4 is a team object, not the character. Live.
* **`h_17a554bf`** (placement attribute) is bit 4 of the object's +0x70 flags (parser
  FUN_00156390). The only code that reads it (getter 0x10f8e0, vtable slot +0x124) is
  FUN_000da8f0, which gives the object a `follow_cam_%X` camera, and FUN_000db3e0, which
  deletes that camera with the object. It marks the characters a player can take control of,
  not "active at start". Static; see #102.
* **Enemies and the squad come from spawn-triggers** (vtable 0x395838, parser FUN_00209250,
  actions FUN_00209ac0, spawn FUN_002099e0). Static, from the code: a trigger clones its
  template object (+0xcc, the `<h_0884288c objects=...>`) at its spawn point (+0xb4) when it
  gets `TRIG_ACT_OPERATE` (1) while armed (+0xe4 bit 3), at once or after its delay
  `h_eab6952f` (+0xe0); the copy is kept at +0xc4; `TRIG_ACT_SPAWN_RESET` (36) re-arms it when
  `h_0f95f576` (+0xe4 bit 2) is set. Live, e40: the enemy `character-object`s of the level
  file exist at the start as inert templates; the four squad members are the copies of
  triggers at their start points (Brutus's trigger is at (−14.0, 37.1, 160.7): 0.3 m from him
  horizontally but 10.9 m above where he stands, y 26.2; presumably he drops to the ground, not observed),
  which is where #102's squad start comes from. Live too: firing a trigger by moving its spawn
  point and setting the armed and pending bits with an almost spent timer spawns its template
  there (the trainer's `spawn`).
* **The debug features are not in the retail game.**
  * `common/debug-config-xbox.xmb` is never loaded: FUN_001e8160 stores its name in the
    `std::string` at 0x400438, and the only other code touching that string is its static
    destructor (0x311070). None of its own attribute hashes (`debug-interface`'s
    `h_ffd5618b` … `h_f0489848`, `enable-console`, `debug-interface`, `readout`, the root's
    attributes) occurs anywhere in default.xbe.
  * The `VB_DEBUG_*` actions (43-60, enum table 0x3bf398) have no reader. Pad bindings are
    16-byte records {pad input, VB, mode, threshold} read by FUN_001e7b30 (eight blocks in RAM,
    from `game-options-en.xml`'s `<action>`). Live: rebinding the black button to VB 12
    (next item) changed the item box, but bound to each of VB 43-60 it did nothing visible.
    `TRIG_SIG_DEBUG_MENU` (19) exists only in the trigger-signal name table.
* **`take.py`** runs one staged take from a JSON spec: load a snapshot, apply trainer edits,
  record picture and sound with `record.py` while a timed pad sequence plays, and append the
  take's `notes.md` entry (snapshot, edits, each input's time from the first video frame, xemu
  version, A/V offset). First take: `todo/79-roller-grenade/take-dev01-…` (an enemy spawned
  and frozen 10 m ahead, the squad frozen; the Roller went off by it 2.75 s after LT).
