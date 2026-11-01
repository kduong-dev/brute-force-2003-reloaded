#!/usr/bin/env bash
# Build and run the demo (bf_play) from the front end.
# Extra arguments go to bf_play; environment variables (BF_MAP=e34, BF_CHARACTER=flint, ...)
# pass through, e.g. `BF_MAP=e34 ./play.sh`.
# RUST_BACKTRACE=1 by default, so a panic prints where it came from (#349).
set -euo pipefail
cd "$(dirname "$0")/viewer"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
exec cargo run -q --bin bf_play -- "$@"
