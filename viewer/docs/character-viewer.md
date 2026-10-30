# Character viewer (`bf_viewer`)

`bf_viewer` decodes everything itself (ports of the Python tools in the repo root):

| Module | Python original | What |
|---|---|---|
| `src/bf/hash.rs` | `xmb_tool.name_hash` | name hash (CRC32 variant, game's own table) |
| `src/bf/archive.rs` | `tarfile` | level `.tgz` archives |
| `src/bf/bxml.rs` | `tools/xmb_tool.py` | BXML documents + BXSD schemas (LZ token stream, typed values) |
| `src/bf/texture.rs` | `tools/tex_tool.py` | texture index + DXT1/3/5 and swizzled formats -> RGBA |
| `src/bf/character.rs` | `tools/char_render.py` | skeleton, skinned meshes (incl. NV2A push buffers), skin materials, animation channels |

It loads `common.tgz` (all BXML decoded against the schemas) in about 0.3 s. Squad skins
live in the level archives, which are opened on demand for their textures only: with no level
loaded, the first by name (e01's tan outfit). Each level carries its own outfit, which bf_play
and bf_level load with the level (character-types.md, "The squad's outfits", #284).

Check against the Python tools:

```
cargo run -- --dump brutus 20 0.5     # counts, bind check, bone positions for anim 20 at 0.5 s
```

All four characters match `tools/char_render.py` exactly (bone counts, triangles, animations) and
bone positions agree to ~1e-6.

## Controls

| Key | Action |
|---|---|
| 1-4 / Tab | switch character (Brutus, Flint, Hawk, Tex) |
| Right / Left | next / previous animation |
| PageDown / PageUp | jump 10 animations |
| Q / E | previous / next face: the game's rest face (the default), the raw bind face, then each clip of the face set by name (`Sc_base` ... `Sc_blink`) |
| L | toggle game shading vs the old flat look |
| Space | pause / resume |
| `[` / `]` | half / double speed |
| Left-drag, wheel | orbit, zoom |
| F | reset camera |
| G | toggle ground plane |

Animation numbers match `python tools/char_render.py anims <name>`.

## Notes

* The camera follows the skeleton root, so animations with root motion stay in frame. Skinned
  meshes have frustum culling disabled (their bounding box is the bind pose's).
* Shading: skins are `BF_CS_rt` ("Color-Specular") materials whose texture alpha is a specular
  mask. The skin doubles as Bevy's `specular_texture` (alpha -> reflectance, the
  KHR_materials_specular convention; needs the `pbr_specular_textures` feature), a generated
  roughness map makes masked areas glossy, and the material's constants set tint / strength /
  gloss (read as h_16e7f952 colour, h_e1664805 level, h_e7604658 glossiness: inferred from the
  values, the names are hashed). Lighting is a warm key with shadows, cool fill and a rim light
  over a dim ambient. `BF_FLAT=1` (or L) gives the previous flat look for comparison.
* Faces (#255, [faces.md](faces.md)): human faces are modelled mid-speech, so the game always writes the
  face bones (no body clip moves them). The viewer starts on **the game's rest face**: `Sc_base`,
  the base pose of the `<name>face` set (duration 0, one key), on the face bones, and the blink
  clip's last key on the lids (what a face controller shows once its first blink is over). Q/E
  cycles through that, "raw bind" and every clip of the set, each laid on the face bones at its
  first key (a viseme is two keys over 0.033 s, an expression one pose held 1 s), by name
  (`Sc_roar`, `Sc_angry` ...). `Sc_blink` is the one clip that is an animation (0.333 s, the lids).
* Test hooks: `BF_CLOSEUP=1` frames the head (and follows the head bone as it is posed: Brutus's hunched head hangs
  low and forward and wants a larger `BF_CAMERA_DISTANCE` and a `BF_CAMERA_PITCH`). `BF_SCREENSHOT=out.png` (plus optional `BF_CHARACTER=0-3`, `BF_ANIM=<n>`) plays for
  2 seconds, saves a screenshot and exits. `BF_CHARACTER` also sets the starting character.
  `BF_DATA_DIR` overrides where the `.tgz` archives are read from. `BF_FRAMES=<n>` captures n
  frames (`out_000.png` ...) at a fixed 15 fps from the clip start, for GIFs; `BF_NO_HUD` hides
  the overlay and `BF_CAMERA_DISTANCE` sets the starting camera distance; `BF_CAMERA_YAW` / `BF_CAMERA_PITCH` (radians, yaw relative to
  the front) and `BF_CAMERA_TARGET_Y` aim it, and `BF_FACE=game|none|<n>` picks the face: the game's rest face (the default), the raw bind face, or face clip `n`
  (the HUD's "face clip 12/24": n is 11 for the squad's `Sc_base`).
