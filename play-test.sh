#!/usr/bin/env bash
# Build and run the demo on the test map (bf_play --test).
# Extra arguments go to bf_play; environment variables pass through, e.g.
# `BF_TEST_FULL_SQUAD=1 ./play-test.sh` or `BF_MAP=sdm_e34 ./play-test.sh` (the test tools on a level).
# RUST_BACKTRACE=1 by default, so a panic prints where it came from (#349).
# Everything runs in a subshell, so the script is safe to source or run from an editor: it never
# changes or closes the calling terminal.
(
  cd "$(dirname "${BASH_SOURCE[0]:-$0}")/viewer" || exit 1
  RUST_BACKTRACE="${RUST_BACKTRACE:-1}" cargo run -q --bin bf_play -- --test "$@"
)
