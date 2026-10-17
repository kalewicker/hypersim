#!/bin/sh
# Run the preserved LK workspace from anywhere; keep relative rule paths local.
set -eu
project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_root/experiments/lk"
# Keep package names and build products isolated from the main workspace.
CARGO_TARGET_DIR="$project_root/experiments/lk/target"
export CARGO_TARGET_DIR
exec cargo "$@"
