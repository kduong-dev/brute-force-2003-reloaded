*Part of [levels/](README.md), the former page `levels.md`.*

## Other maps

Every squad deathmatch map loads (`BF_MAP=sdm_e01|sdm_e10|sdm_e13|sdm_e34|sdm_e40|sdm_m03|sdm_m07`,
or `cargo run --bin bf_level -- <map>`). What it took:

* **Terrain grid per level.** The terrain vertex shader splits a vertex's first short into row
  and column by 8 × the terrain's width in blocks + 1, and scales both by the cell size
  (`Terrain h_e023cdc9`: 1, 2 or 4 m). Blocks are 16 cells.

  | Map | Blocks per side | Cell |
  |---|---|---|
  | sdm_e34 | 8 | 2 m |
  | sdm_e01, sdm_e10 | 16 | 1 m |
  | sdm_e13 | 8 | 4 m |
  | sdm_e40 | 16 | 2 m |
  | sdm_m03 | 19 | 2 m |

  The terrain layer masks span the whole grid.
* **Baked terrain light.** Each terrain block carries 16 × 16 cell values (`blocks
  h_e2d79bad`, 0–255): the terrain's baked lighting, cliff shadows included. Terrain is drawn as
  texture × that light. The orientation (blocks row by row, x and z increasing) was found by
  correlating it with the terrain's slope lighting.

  The level's dynamic terrain lights are dim; sdm_e40's key is 0.2, low in the sky. The baked
  light is what the map-select previews show: bright sdm_e40 and sdm_e10, night sdm_e01.
