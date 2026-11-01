#!/usr/bin/env bash
# Build and run the demo (bf_play) from the front end.
# Extra arguments go to bf_play; environment variables (BF_MAP=e34, BF_CHARACTER=flint, ...)
# pass through, e.g. `BF_MAP=e34 ./play.sh`.
# RUST_BACKTRACE=1 by default, so a panic prints where it came from (#349).
# Everything runs in a subshell, so the script is safe to source or run from an editor: it never
# changes or closes the calling terminal.
(
  cd "$(dirname "${BASH_SOURCE[0]:-$0}")/viewer" || exit 1
  RUST_BACKTRACE="${RUST_BACKTRACE:-1}" cargo run -q --bin bf_play -- "$@"
)
