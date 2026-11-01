#!/usr/bin/env bash
# Build and run the demo on the test map (bf_play --test).
# Extra arguments go to bf_play; environment variables pass through, e.g.
# `BF_TEST_FULL_SQUAD=1 ./play-test.sh` or `BF_MAP=sdm_e34 ./play-test.sh` (the test tools on a level).
# RUST_BACKTRACE=1 by default, so a panic prints where it came from (#349).
set -euo pipefail
cd "$(dirname "$0")/viewer"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
exec cargo run -q --bin bf_play -- --test "$@"
