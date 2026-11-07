*Part of [maps/](README.md), the former page `maps.md`.*

* **Pickups.** They're stored at their spawn height in the level file; they now settle onto the
  floor below.
* **Compound joints.** A compound object's joints carry a rotation (a w-x-y-z quaternion),
  applied to the part (the wall sets turn pieces by 90° and 180°). A part sits at the joint's
  parent-point + child-point, both in the parent's frame; the rotation turns only the part about
  its own origin. Bulgar's compound `h_0baf091a` (hangar, bunk rooms: 19 pieces) is the one placed
  object with non-zero child-points, and only this reading puts every piece inside the building:
  read as parent-point − rotation × child-point, its roof bays landed outside the hangar and its
  turned bunks went through the walls (issue #12). `BF_OVERLAP_LOG=1` (bf_level) lists large
  objects whose boxes overlap much.
* **Winding.** Object triangles are wound to face along their stored normals, so a mirrored mesh
  isn't culled from the front.
* **The gate (h_0792d16a) and texture address modes.** Its two leaves (h_e783bd32 → mesh
  h_e01d9aa6, h_fcd25d86 → h_178bc697) are the same panel shifted 0.996 m, with matching UVs
  and no joint rotation. The mirroring is in the texture's sampling: each leaf runs its
  texture u 0..2 across, and the texture entry's `address-u` / `address-v` (h_1c82a17b /
  h_078bf0c1) is `TAM_MIRROR` (h_18712765), so the second half is the first mirrored and each
  leaf shows one symmetric brace. The other modes are `TAM_WRAP` (h_e3012d9e, most textures)
  and `TAM_CLAMP` (h_e041e0b9). Every level and model texture is now sampled by its own modes
  (`Game::texture_address`); 24 levels use mirroring somewhere.
* **Invisible or glowing materials.**
  * Power-ups (Brute / Force / Health / Stamina) use a placeholder cube whose material no level
    defines; it isn't drawn.
  * Untextured glow materials (type h_f539fe8c, e.g. the medkits' shell) are drawn as their
    glow colour at the wrapper material's opacity.
* **Hits.** Shots, grenades, decals, the memory chip and the follow camera use the same collision.
