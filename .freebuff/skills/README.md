# Freebuff skills

The project's agent roles, ported from `.claude/agents/` so Freebuff can load them as
skills:

| Skill | Role |
|---|---|
| `analyst` | Finds what the original game does in its data and `default.xbe`; writes leads, shot lists, code answers and ticket drafts. |
| `developer` | Implements one ticket in the demo, from the spec and the game data. |
| `reviewer` | Read-only review of a branch, PR or diff. |
| `tester` | Measures xemu footage into a spec, then checks the demo against it. |
| `xemu` | Records the original game in xemu. |

Each is `.freebuff/skills/<name>/SKILL.md`, with the role's Claude Code frontmatter reduced to
the `name` and `description` a skill takes; the model, effort and tool lists do not carry
over. The instructions themselves are the same, with cross-references moved from
`.claude/agents/` to these paths.

The hooks in `.claude/settings.json` are Claude Code's and do not run here: the game-data
commit guard and the Windows finish toast are not replicated. The game-data rule they enforce
is still in `CLAUDE.md` (`Brute Force/`, `decompiled/` and `todo/` are git-ignored and must
never be committed), and the developer and reviewer skills repeat it.

When a role changes in `.claude/agents/`, mirror the change here.
