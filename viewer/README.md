# Brute Force character viewer (Rust / Bevy)

Four programs:

| Binary | What | Run |
|---|---|---|
| `bf_viewer` (default) | character / animation viewer, reads the **original game files** (`../Brute Force/data/*.tgz`) | `cargo run` |
| `bf_play` | **playable demo**: run around as Brutus, Flint, Hawk or Tex with the game's own locomotion | `cargo run --bin bf_play` (the test map: `cargo run --bin bf_play -- --test`) |
| `bf_level` | level viewer: terrain, placed objects, sky and fog of a level (default Battle of Bulgar) | `cargo run --bin bf_level -- [level]` |
| `glb_viewer` | viewer for the exported `../decompiled/characters/*.glb` | `cargo run --bin glb_viewer` |

How everything works, one page per area (each says where its values come from, its test hooks
and how it was checked):

| Page | What |
|---|---|
| [docs/play.md](docs/play.md) | `bf_play`: keys, the test map, health and HUD, locomotion, weapons, sounds, the general test hooks |
| [docs/maps.md](docs/maps.md) | playing on a level: spawning, collision, doors and gates, HUD text, movement, music, shots, look, power-ups |
| [docs/squad.md](docs/squad.md) | the squad: follow AI, standing on the floor, scopes, hand-over, hits, deaths, the death camera |
| [docs/grenades.md](docs/grenades.md) | grenades: throwing and placing, blasts and damage, Gas, Sentry, Light, Sonic, Energy |
| [docs/pickups.md](docs/pickups.md) | health pickups and the item box |
| [docs/scenery.md](docs/scenery.md) | interactive scenery: barrels, racks and crates that break, explode and chain |
| [docs/menu.md](docs/menu.md) | the front end: title, menus, mission select, movies |
| [docs/levels.md](docs/levels.md) | `bf_level` and how levels are drawn: terrain, materials, liquids, lights, sky, each map's quirks |
| [docs/character-viewer.md](docs/character-viewer.md) | `bf_viewer`: the format readers, its keys and shading notes |
| [docs/xemu.md](docs/xemu.md) | the original game in xemu: what the trainer and takes rely on in `default.xbe` |

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
