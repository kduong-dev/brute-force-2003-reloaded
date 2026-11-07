# Level viewer (`bf_level`)

*This folder was the page `levels.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../levels.md` is now a pointer to them.*

`cargo run --release --bin bf_level -- [level]` draws a level from the original archives:
terrain, every placed object (game objects, pickups) at its transform, the sky, and the level's
fog, with the power-ups' icons and the level's placed effect objects running (its flies,
embers, fires, steam and dust: maps.md "Placed effect objects", #258). The spawn-triggers'
pickup templates are among them, icons and all: every pickup of the multiplayer maps (sdm_e34's
54, its four power-ups) shows where the level places it, though in the game, and in bf_play,
it's out of the world and its trigger's copies appear at the trigger instead ([pickups.md](../pickups/pickups-from-spawn-triggers.md),
"Pickups from spawn-triggers", #278). The viewer runs no triggers. Its other templates
(characters, effects: [triggers.md](../triggers/spawn-triggers.md), "Spawn-triggers", #230) aren't drawn. The default is `sdm_e34`,
the squad deathmatch map "Battle of Bulgar" (SDM Level 5 in
campaign-bf.xml). Right-drag looks around, W/A/S/D/Q/E fly (Shift for faster), 1-9 jump to the
level's own flyby cameras and 0 to an overview, F toggles fog, P prints the view. Test hooks:
`BF_LEVEL_CAMERA=<n>`, `BF_VIEW=x,y,z,yaw,pitch`, `BF_NO_FOG=1`, `BF_SCREENSHOT=<file.png>`,
`BF_LEVEL_DUMP=1` (placements, untextured materials, idle effects, placed effect objects),
`BF_NO_PLACED_FX=1` (no placed effect objects), `BF_PART_DUMP=<hex
archetype>` (each part's geosets: offset, bounds, normals, winding, UV direction; then exits),
`BF_TEX_LOG=1|squad` (where each texture came from, below), `BF_SKY_CHECK=1` (every level's sky
layers, blended or opaque, then exits: "Sky", below).
In bf_level and bf_play: `BF_NO_DETAIL=1` (the terrain without its detail texture),
`BF_NO_BUMP=1` (the terrain without its bump map), `BF_ANISOTROPY=<1-16>` (the world textures'
anisotropy; the game's is none), `BF_SKY_TIME=<s>` (the sky's clock started that far into the
level), `BF_GLARE=0` (no glare) and `BF_GLARE_DEBUG=1|2` (the glare's keys, or its blur alone),
all below.

**Sections**

* [Terrain: vertices, layers, the detail (#257) and the bump map (#271)](terrain.md)
* [Which copy of a texture (#284)](which-copy-of-a-texture.md)
* [Which level's data (#338)](which-levels-data.md)
* [Mip levels and filtering (#257)](mip-levels-and-filtering.md)
* [Level transforms, object types and the glow material](placements-and-materials.md)
* [Other maps: the squad deathmatch maps, the terrain grid per level, the baked terrain light](other-maps.md)
* [Liquid surfaces (#4)](liquid-surfaces.md)
* [Liquids aren't solid; what a liquid does to a character (#152)](liquids-and-characters.md)
* [What a liquid plays (#5)](what-a-liquid-plays.md)
* [Terrain hue; light objects (#143)](terrain-hue-and-light-objects.md)
* [Point lights](point-lights.md)
* [Effect lights (#272)](effect-lights.md)
* [Grass; shadows; what #143 checked](grass-shadows-and-light-checks.md)
* [Sky: drawn first (#290), turning (#199), sdm_m07's background](sky.md)
* [Known gaps](known-gaps.md)
