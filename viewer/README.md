# Brute Force character viewer (Rust / Bevy)

Four programs:

| Binary | What | Run |
|---|---|---|
| `bf_viewer` (default) | character / animation viewer, reads the **original game files** (`../Brute Force/data/*.tgz`) | `cargo run` |
| `bf_play` | **playable demo**: run around as Brutus, Flint, Hawk or Tex with the game's own locomotion | `cargo run --bin bf_play` (the test map: `cargo run --bin bf_play -- --test`; the map editor: `-- --edit`) |
| `bf_level` | level viewer: terrain, placed objects, sky and fog of a level (default Battle of Bulgar) | `cargo run --bin bf_level -- [level]` |
| `glb_viewer` | viewer for the exported `../decompiled/characters/*.glb` | `cargo run --bin glb_viewer` |

How everything works, one page per area (each says where its values come from, its test hooks
and how it was checked):

| Page | What |
|---|---|
| [docs/play.md](docs/play.md) | `bf_play`: keys, the test map, health and HUD, locomotion, weapons and shots (fire timing, pellets, accuracy, holes, tracers, casings), missiles (the Therm Sweeper's MIRV, blasts, scorch and camera shake), damage (the difficulty step, the hit parts and their factors, the combat log), ammo, reload and the crosshair, sounds, the general test hooks |
| [docs/maps.md](docs/maps.md) | playing on a level: spawning, collision, doors and gates, HUD text, movement, music, shots, look, power-ups |
| [docs/triggers.md](docs/triggers.md) | the level's triggers, run (#220): events, entries and the base's actions, routers, timers, calculators, spawn-triggers (templates, copies, the squad's start, the member check), detectors (plates' boxes, filters, occupancy), anim-triggers (the doors), destroyers, gamevar and score triggers (#221), object events; what later steps run; `BF_TRIGGER_LOG`, `BF_TRIGGER_FIRE` |
| [docs/campaign.md](docs/campaign.md) | the campaign's state (#221): the game flags (default-gameflags), the missions (campaign-bf), the squad's save-game slots and member flags, the cash (score-triggers, death awards, the difficulty's scaling), the MISSION OBJECTIVES menu (I) and the clock stopping under it; `BF_FLAG_LOG`, `BF_FLAG_SET`, `BF_CASH_LOG`, `BF_TEST_OBJECTIVES` |
| [docs/squad.md](docs/squad.md) | the squad: follow AI, standing on the floor, scopes, hand-over, hits, deaths, the death camera |
| [docs/character-types.md](docs/character-types.md) | character types from the data (#128): every type by its `<base name>`, motion sets and their clips, placements and teams, flesh; what play keys by type; `BF_DUMP_TYPES` |
| [docs/enemies.md](docs/enemies.md) | the enemy species' own behaviour (#129, the Shrikes first): a placement's gun, the MOTION table's clips and the type's speeds, hit reactions by personality, the ragdoll's bodies and joint limits, the death line, green blood and the game's decal rule, the deathmatch stamina shield; `BF_TEST_NPC_MOTION`, `BF_ENEMY_LOG` |
| [docs/skills.md](docs/skills.md) | special skills: stamina, the skill toggle (V), its sounds, chatter and HUD (the blue bar's colours, trail and blink, the skill's name), the interface the four skills plug into, Tex's Berserker (his second gun, its fire, the berserk motions, what ends it), Brutus's Spirit of Vengar (his regeneration, the damage he takes, the charge, the Vengar view: the vision lights, the world pass, the heat figures and their pulse, the red fade), Hawk's Stealth (her invisibility controller and her skin's fade, her refraction: the game's invisibility material over a copy of the picture, its wobble and a shot's flash, the costs of a shot and an item, who sees her: the AI's sight and the crosshair, the AI's hold fire, her footsteps) and Flint's Advanced Targeting (the lock box, the targeting marker, the sniper motions, her doubled and scoped rolls, the rifle's range) |
| [docs/grenades.md](docs/grenades.md) | grenades: throwing and placing, blasts and damage, Gas, Sentry, Light, Sonic, Energy |
| [docs/pickups.md](docs/pickups.md) | health pickups and the item box |
| [docs/scenery.md](docs/scenery.md) | interactive scenery: barrels, racks and crates that break, explode and chain |
| [docs/testtools.md](docs/testtools.md) | the test map's developer tools: free camera and teleport, NPCs (friend / enemy, dummy / fight), placing objects (on the ground or in the air), any level's buildings and static scenery (drawn and solid as in their levels), skies, music, switching levels |
| [docs/editor.md](docs/editor.md) | the map editor (#190): its edit mode out of the gunplay (`--edit`, F5), the edit camera and controls, play from here and back, what carries over, the tool registry and the list of placed things the later editing tickets build on |
| [docs/menu.md](docs/menu.md) | the front end: title, menus, mission select, movies |
| [docs/briefing.md](docs/briefing.md) | a mission's briefing movies (#225): the zones' movie lists, the subtitles' timing, typing, layout, font and markup, skipping, the entry point for the mission flow, converting the movies; `BF_MOVIES` |
| [docs/levels.md](docs/levels.md) | `bf_level` and how levels are drawn: terrain, materials, liquids, lights, sky, each map's quirks |
| [docs/character-viewer.md](docs/character-viewer.md) | `bf_viewer`: the format readers, its keys and shading notes |
| [docs/xemu.md](docs/xemu.md) | the original game in xemu: what the trainer and takes rely on in `default.xbe` |
| [docs/code-map.md](docs/code-map.md) | code map: the `default.xbe` functions and addresses the project relies on, by area: what each does, how sure, where it's used |

A change updates the page for its area (and this table if it adds a page).

## Building

* First build compiles Bevy (about 7 minutes); later builds take under a minute. The dev profile
  optimises dependencies, so `cargo run` is smooth; `--release` would trigger a second full build.
* Bevy is pinned to 0.16.1 in `Cargo.toml`; Bevy breaks its API each release, so upgrade
  deliberately.
* Toolchain: `stable-x86_64-pc-windows-gnu` plus WinLibs MinGW-w64 (provides the `dlltool`/`as`
  that the bundled MinGW kit lacks). dlltool fails on paths with spaces (like "XBE Mod"): if
  the project's path has one, send the build output elsewhere in a local, uncommitted
  `viewer/.cargo/config.toml`, e.g. `[build] target-dir = "C:/Users/<you>/.cargo/target/bf_viewer"`.
