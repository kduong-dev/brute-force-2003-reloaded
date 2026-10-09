"""PreToolUse guard: never commit the game data.

`Brute Force/`, `decompiled/` and `todo/` hold the original game's files, data extracted from
them and the xemu recordings. They are git-ignored and must never be committed (CLAUDE.md).
This blocks a shell command that would stage or commit any of them:
- `git add` that names one of those folders (with or without -f), or `git add -f`/`--force`
  with paths under them;
- `git commit` while the index already holds a file under them (checked with
  `git diff --cached --name-only`).

Reads the hook payload (JSON) on stdin; exit 2 with a reason on stderr blocks the call.
"""
import json
import re
import subprocess
import sys

PROTECTED = ("Brute Force/", "decompiled/", "todo/")


def protected(path: str) -> bool:
    p = path.strip().strip("'\"").replace("\\", "/")
    p = re.sub(r"^\./", "", p)
    return any(p == d.rstrip("/") or p.startswith(d) or ("/" + d) in p for d in PROTECTED)


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0
    cmd = (payload.get("tool_input") or {}).get("command") or ""
    if "git" not in cmd:
        return 0
    bad = []
    # only the arguments of each `git add` (up to the next ; & | or line break), not the rest of
    # the command, so commit messages or comments that mention the folders don't trip it
    # (git's own options before `add` may take an argument: -C <path>, -c <name=value>)
    for seg in re.findall(r"\bgit\b(?:\s+(?:-[Cc]\s+(?:\"[^\"]*\"|'[^']*'|\S+)|-\S+))*\s+add\b([^;&|\n]*)", cmd):
        for d in PROTECTED:
            if d.rstrip("/") in seg.replace("\\", "/"):
                bad.append(d)
    if re.search(r"\bgit\b[^;&|\n]*\bcommit\b", cmd):
        cwd = payload.get("cwd") or None
        try:
            staged = subprocess.run(["git", "diff", "--cached", "--name-only"], cwd=cwd,
                                    capture_output=True, text=True, timeout=20).stdout.splitlines()
        except Exception:
            staged = []
        bad += [f for f in staged if protected(f)]
    if bad:
        sys.stderr.write(
            "Blocked: this would stage or commit git-ignored game data ("
            + ", ".join(sorted(set(bad))[:10])
            + "). Brute Force/, decompiled/ and todo/ must never be committed (CLAUDE.md). "
            "Unstage them (git restore --staged <path>) and commit only source files.\n")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
