# Brute Force (2003) Reloaded

An unofficial, fan-made rebuild of **Brute Force** (Digital Anvil / Microsoft Game Studios,
Xbox, 2003): a playable demo in Rust and [Bevy](https://bevyengine.org) that loads the original
game's own files, plus the tools used to take those files apart.

This project is not affiliated with or endorsed by Microsoft. It contains no game data: you need
your own copy of the game.

## What's here

| Path | What it is |
|---|---|
| [`viewer/`](viewer/) | The Rust / Bevy crate: the demo and the viewers (see [Programs](#programs)). |
| [`viewer/docs/`](viewer/docs/README.md) | How everything works, one page per area: what the game does, where each value comes from, the test hooks, and how it was checked. The [code map](viewer/docs/code-map.md) lists the `default.xbe` functions the project relies on. |
| [`tools/`](tools/) | Python tools for the game's formats and the decompilation, and the Ghidra scripts (see [Tools](#tools)). |

## Getting started

1. Copy the game's disc contents to `Brute Force/` next to this README, so that
   `Brute Force/data/common.tgz` exists.
2. Build and run the demo:

   ```
   cd viewer
   cargo run --bin bf_play
   ```

Some features (the menu's background movie, the opening movies, the credits, the briefings) need
frames converted once from the game's Bink movies with ffmpeg. The commands are in
[menu.md](viewer/docs/menu.md) and [briefing.md](viewer/docs/briefing.md). Extracted and
converted files go in `decompiled/`, which stays out of the repository
(`python tools/build_decompiled.py` regenerates most of it).

### Building

* The first build compiles Bevy (about 7 minutes); later builds take under a minute. The dev
  profile optimises dependencies, so `cargo run` is smooth; `--release` would trigger a second
  full build.
* Bevy is pinned to 0.16.1 in `viewer/Cargo.toml`. Bevy breaks its API each release, so upgrade
  deliberately.
* Toolchain: `stable-x86_64-pc-windows-gnu` plus WinLibs MinGW-w64, which provides the
  `dlltool` and `as` that the bundled MinGW kit lacks. dlltool fails on paths with spaces (like
  "XBE Mod"): if the project's path has one, send the build output elsewhere in a local,
  uncommitted `viewer/.cargo/config.toml`, e.g.
  `[build] target-dir = "C:/Users/<you>/.cargo/target/bf_viewer"`.

## Programs

Run from `viewer/`.

| Binary | What | Run |
|---|---|---|
| `bf_play` | The playable demo: the front end, the campaign's first missions, the deathmatch and squad deathmatch maps, the squad, weapons, skills and HUD | `cargo run --bin bf_play` (the test map: `-- --test`; the map editor: `-- --edit`) |
| `bf_level` | Level viewer: the terrain, placed objects, liquids, sky and fog of a level (default Battle of Bulgar) | `cargo run --bin bf_level -- [level]` |
| `bf_viewer` (default) | Character and animation viewer, straight from the game's archives | `cargo run` |
| `glb_viewer` | Viewer for characters exported to `decompiled/characters/*.glb` | `cargo run --bin glb_viewer` |

## Tools

Run from the repository root, e.g. `python tools/xmb_tool.py --help`; each has its usage at the
top of the file.

| Tool | What it does |
|---|---|
| `tools/xmb_tool.py` | Binary XML (`.xmb`) and schema (`.xsb`) decoder, and the name hash; recovered names are in `tools/xmb_names.txt` |
| `tools/xbe_tool.py` | `default.xbe`: header, sections, XDK libraries, kernel imports, disassembly |
| `tools/tex_tool.py` | Texture libraries (`.tex` + `textures-*.xmb`) to PNG or DDS |
| `tools/xwb_tool.py` | XACT wave banks (music, sounds, voices): list, extract to WAV, replace an entry, rebuild |
| `tools/ale_tool.py` | ALE effect libraries (`effects-<level>.ale`): effects and their nodes |
| `tools/char_render.py` | Renders a character playing an animation; `gltf` exports it with `tools/gltf_export.py` |
| `tools/build_decompiled.py` | Regenerates `decompiled/` from the disc in `Brute Force/` |
| `tools/fn.py` | Prints decompiled functions from `decompiled/xbe/ghidra/` by address or name |
| `tools/xbe_peek.py` | Reads floats, ints or dwords from `default.xbe` by virtual address |
| `tools/ghidra_scripts/` | Ghidra headless scripts: `ExportDecomp.java` writes every function as C; `CreateFunctions.java` creates the functions in `call_targets.txt` that the analysis missed |
| `tools/run_ghidra.sh`, `tools/run_ghidra_pass2.sh` | The headless Ghidra runs; their output goes to `decompiled/xbe/ghidra/` |

Ghidra itself (12.0.3, with the XboxDev ghidra-xbe extension; Java 21 or later) and its project
are installed locally under `tools/` and kept out of the repository.

## Licence

The code is MIT licensed (see [LICENSE](LICENSE)). Brute Force and its assets belong to their
owners.
