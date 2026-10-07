# Brute Force (2003) Reloaded

An unofficial, fan-made rebuild of **Brute Force** (Digital Anvil / Microsoft Game Studios,
Xbox, 2003): a playable demo in Rust and [Bevy](https://bevyengine.org) that loads the original
game's own files, plus the tools used to take those files apart.

This project is not affiliated with or endorsed by Microsoft. It contains no game data: you need
your own copy of the game.

## What's here

| Path | What it is |
|---|---|
| [`viewer/`](viewer/) | The Bevy demo (`bf_play`: front end, deathmatch and squad deathmatch maps, squad, weapons, HUD) and level viewer (`bf_level`). Its [README](viewer/README.md) documents everything found about the game's formats and behaviour. |
| `*_tool.py` | Python tools for the game's formats: XBE executable (`xbe_tool.py`), binary XML (`xmb_tool.py`), textures (`tex_tool.py`), wave banks (`xwb_tool.py`), effects (`ale_tool.py`). |
| [`tools/`](tools/) | Ghidra headless scripts for the decompilation. |

## Getting started

1. Copy the game's disc contents to `Brute Force/` next to this README (so that
   `Brute Force/data/common.tgz` exists).
2. Build and run the demo:

   ```
   cd viewer
   cargo run --bin bf_play
   ```

Some features (the menu's background movie, the opening movies, the credits) need files
converted once from the game's Bink movies with ffmpeg; `viewer/README.md` has the commands.
Extracted and converted files go in `decompiled/`, which stays out of the repository.

## Licence

The code is MIT licensed (see [LICENSE](LICENSE)). Brute Force and its assets belong to their
owners.
