# Brute Force (Xbox, 2003) reloaded

A reimplementation of Brute Force built on the original game's files. It has Python format
tools at the root and a Rust/Bevy viewer and playable demo in `viewer/`. Tickets are GitHub
issues on `kduong-dev/brute-force-2003-reloaded`.

## Layout

| Path | What |
|---|---|
| `viewer/` | Rust + Bevy 0.16 crate: `bf_play` (the demo), `bf_level`, `bf_viewer`, `glb_viewer`. `viewer/README.md` indexes `viewer/docs/`, the reference for how everything works (one page per area) |
| `viewer/src/bf/` | format readers: BXML (`bxml.rs`), levels, characters, textures, sounds |
| `viewer/src/bin/play*.rs` | the demo, one module per feature (`play_hud`, `play_pickups`, `play_grenade`, `play_text`, ...) |
| `xmb_tool.py`, `xbe_tool.py`, `xwb_tool.py`, `tex_tool.py`, `ale_tool.py` | format tools: BXML decoder, XBE, music banks, textures, ALE effects |
| `Brute Force/`, `decompiled/`, `todo/` | game data, extracted data and reference captures. **Git-ignored. Never commit them or copy them anywhere** |
| `tools/` | Ghidra scripts (the Ghidra install and its projects are ignored) |

## Building and running

- The GNU toolchain's dlltool fails on paths with spaces ("XBE Mod"), so build outside the
  repo. `viewer/.cargo/config.toml` (ignored, per machine) sets
  `target-dir = "C:/Users/Kevin/.cargo/target/bf_viewer"`. A git worktree has no copy of it:
  set `CARGO_TARGET_DIR` to a directory of its own, without spaces, e.g.
  `C:/Users/Kevin/.cargo/target/bf_<branch>`.
- Game data is found at `../Brute Force/data` from `viewer/`. A worktree has no copy of it:
  set `BF_DATA_DIR="C:/Users/Kevin/projects/github/XBE Mod/Brute Force/data"`.
- `cd viewer && cargo build --bins` must finish with **no errors and no warnings**.
- A full build takes minutes. Agents that share a target dir queue on its lock.
- Builds go through sccache (machine-wide `~/.cargo/config.toml`, cache on `D:\sccache`), so a
  new worktree's build reuses the compiled dependencies. Leave it on.
- **xemu runs at above-normal priority**, so builds and demo runs don't make the game stutter
  (which spoils footage): the `xemu` agent raises its own xemu process when it starts it.
- **While an xemu session is recording, run at most one build** at a time; queue the others.

## Verifying in the game (required for any visible change)

The demo has environment-variable test hooks; each is listed on its area's page in `viewer/docs/`. The ones
used most:

```sh
BF_MAP=sdm_e34 BF_TEST_GOTO=x,z,x2,z2[,height] BF_CAPTURE=<dir> BF_CAPTURE_FRAMES=<n> cargo run -q --bin bf_play
```

- `BF_TEST_GOTO`: start at (x, z) and run toward (x2, z2). A goal equal to the start stands
  still.
- `BF_CAPTURE`: saves every frame as a PNG, at a fixed 15 fps.
- Feature hooks: `BF_TEST_USE`, `BF_TEST_MEDKIT=<s>`, `BF_TEST_HEALTH=<hp>`,
  `BF_TEST_FIRE`, `BF_TEST_KILL`, `BF_MENU_GO`.
- Logging: `BF_PICKUP_LOG`, `BF_DOOR_LOG`, `BF_SOUND_LOG`, `BF_MAT_LOG`, `BF_LEVEL_DUMP`
  (`bf_level`), `BF_PART_DUMP=<hex archetype>` (`bf_level`).
- Look at the frames (crop or tile them with PIL) and compare them with the reference captures
  in `todo/`.
- Put scratch output in your scratchpad, never in the repo.

## Team workflow

The agent roles are in `.claude/agents/`:
- `analyst`: reads the game data and default.xbe. It writes a ticket's leads (and the shot
  list when footage is needed) into `todo/<ticket>-<slug>/`, answers the code questions the
  others can't settle, drafts tickets for what they found out of scope, and keeps
  `viewer/docs/code-map.md`. It doesn't build or run the demo.
- `xemu`: plays the original game in xemu and records footage for a ticket, as many takes as
  it needs, into `todo/<ticket>-<slug>/` with a `notes.md`. It may stage shots in a modified
  copy of the game (on D:, never the originals). It runs xemu with its own settings and hard
  disk image, plays through a virtual controller (never the keyboard or mouse) with the
  window on the left monitor, and may run while the user is at the PC.
- `tester`: owns "what the real game does". Before a ticket it measures the footage into a
  spec, `todo/<ticket>-<slug>/spec.md` (or writes a shot list for `xemu`); after, it compares
  the demo with the footage and checks for regressions. Read-only on the source.
- `developer`: implements a ticket in its own worktree, from the spec, the footage and the
  game data.
- `reviewer`: reviews the code (a branch, a pull request or a diff), read-only.

Agents can't start other agents, so **the main session is the project manager and team
lead**: it writes tickets, starts the agents, relays findings, moves the board, and merges.
**The user is the product owner**: they set priorities, judge what looks right, and give the
final yes to merge, push and close tickets.

**Tickets.** The user decides what becomes a ticket. The lead writes it with
`gh issue create`, after checking the game data (or having the `analyst` do it) so the ticket
has leads, using the sections of `.github/ISSUE_TEMPLATE/feature.md`:
- what the real game does;
- leads in the game files;
- acceptance;
- out of scope.

Label it from the existing set (`fidelity`, `gameplay`, `rendering`, `audio`, `combat`, ...).
Epics get `epic` and sub-issues. Other agents never file tickets: they list problems outside
their ticket under "found, not in scope" in their report. The lead asks the user before
filing those, or files them with the `triage` label for the user to accept or close; the
`analyst` can draft them first (duplicates checked, leads gathered) in `todo/drafts/`.

**Order for a ticket:**
1. **Leads.** If the ticket needs research the lead hasn't done, `analyst` writes
   `todo/<ticket>-<slug>/leads.md`, and the shot list if footage is needed.
2. **Footage.** If `todo/<ticket>-*/` doesn't cover the ticket, `xemu` records it, from the
   ticket and the shot list if there is one.
3. **Spec.** `tester` measures the footage into `spec.md`. If something is still missing, its
   shot list goes back to `xemu` (step 2); its code questions go to `analyst`.
4. **Build.** `developer` implements it in its own worktree (`isolation: "worktree"`, with
   `CARGO_TARGET_DIR` and `BF_DATA_DIR` set as above), given the spec and the footage folder.
5. **Check.** `reviewer` and `tester` run in parallel on the result: the code, and the demo
   against the footage.
6. Send the findings back to the same developer (SendMessage) until the reviewer approves and
   the tester passes it. A code question nobody can settle goes to `analyst` meanwhile.
7. The lead summarises the result for the user. On their yes: merge to `main`, push, and the
   commit's `Closes #N` closes the ticket. Then delete the ticket's build folder
   (`CARGO_TARGET_DIR`), its worktree and branch, and the agents' captures for it: each ticket
   leaves about 12 GB of build output and several GB of frames. Keep `todo/` footage.

**The board.** Tickets are tracked on the project "Brute Force Reloaded Project" (#4,
owner `kduong-dev`). Its statuses are Backlog → Ready → In progress → In review → Done. The
lead moves a ticket's card:
- to **In progress** when a developer starts it, or when the review sends it back;
- to **In review** when the reviewer and tester start on it.

Closing the issue moves the card to **Done** by itself: the board's "Item closed" workflow.

```sh
# the ticket's item id on the board
ITEM=$(gh project item-list 4 --owner kduong-dev --limit 500 --format json \
  -q '.items[] | select(.content.number==<N>) | .id')
# set its status (option ids below)
gh project item-edit --project-id PVT_kwHOAje2rc4Bl_qf --id "$ITEM" \
  --field-id PVTSSF_lAHOAje2rc4Bl_qfzhkqXQ0 --single-select-option-id <option>
```

| Status | Option id |
|---|---|
| Backlog | `f75ad846` |
| Ready | `61e4505c` |
| In progress | `47fc9ee4` |
| In review | `df73e18b` |
| Done | `98236657` |

New issues are added to the board automatically. The `gh` login needs the `project` scope
(`gh auth refresh -s project`).

Run at most two heavy agents at once: a build takes minutes and a lot of CPU, and an `xemu`
session uses the GPU and the screen. Heavy means a developer, a tester checking a build, or
`xemu`; the `analyst`, the `reviewer` and a tester writing a spec are light and run beside
them. Two xemu sessions may run side by side, each with its own folder and hard disk copy,
ports, virtual pad (the `xemu` agent: `D:\Emulators\Xbox\agent\`, 4444/1234, an X360 pad, the
left half of the left monitor; a second one: `agent-dev\`, 4445/1235, a DualShock 4 pad, the
right half).

**Effort.** Each role's default effort is in its frontmatter: `analyst`, `reviewer` and
`developer` max (depth there saves review rounds and wrong leads); `tester` xhigh; `xemu` high
(long sessions holding the GPU and screen, where a faster turn frees the slot sooner). The lead
may start a `developer` at high for a small fix (one module, no new mechanics), and `xemu` at
max for shots that need new memory research or tricky staging. Otherwise it leaves the
default.

Relay what agents report faithfully, including what failed or wasn't tested.

## Conventions

- **Faithful to the game.** Values come from the game data (BXML attributes, XBE tables,
  textures, sounds) or are measured from captures. A guess is called a guess in its comment.
- **The code map.** `viewer/docs/code-map.md` indexes the default.xbe functions the project
  relies on (address, what it does, how sure, where it's used). Check it before tracing a
  function, and add a row for each one a change cites.
- Names are 32-bit hashes, written `h_xxxxxxxx` (`bf_viewer::bf::hash::h("name")` for known
  strings). Hashes taken from file names in sound banks are stored byte-reversed.
- **Comments**: a doc comment on every const, struct and fn, saying what it is and *where the
  value came from* (an attribute hash, a capture, an event id). Match the density and voice of
  the surrounding code: plain sentences, no marketing.
- New test hooks are `BF_*` environment variables, documented in their comment and on the
  area's `viewer/docs/` page.
- **Every change updates its area's page in `viewer/docs/`** (and the index in `viewer/README.md` for a new page): controls, the feature's section, and how it was
  verified.
- Commits go on `main` (or a branch to be merged into it) with a short imperative subject and a
  body that explains why. End it with `Closes #N` for each ticket it finishes. Commit or push
  only when asked.
