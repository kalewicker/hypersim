#!/usr/bin/env bash
# v4 experiment driver. Runs a (economy x mu x seed) grid with bounded
# parallelism. Usage: sweep.sh <tag> <seed_pop> <economies...> -- <mus...>
set -u
BIN=./target/release/hg-selfmod
RUNS=crates/hg-selfmod/runs
JOBS=${JOBS:-8}
EXTRA=${EXTRA:-}

TAG=$1; POP=$2; shift 2
ECONS=()
while [ "$1" != "--" ]; do ECONS+=("$1"); shift; done
shift
MUS=("$@")
SEEDS=(1 2 3 7 42 99 123 777)

mkdir -p "$RUNS"
for econ in "${ECONS[@]}"; do
  for mu in "${MUS[@]}"; do
    for seed in "${SEEDS[@]}"; do
      while [ "$(jobs -rp | wc -l)" -ge "$JOBS" ]; do wait -n; done
      name="${TAG}_${econ}_mu${mu}_${POP}_seed${seed}"
      $BIN phaseB --seed="$seed" --economy="$econ" --seed_pop="$POP" --mu="$mu" \
          $EXTRA --out="$RUNS/$name" > "$RUNS/$name.log" 2>&1 &
    done
  done
done
wait
echo "sweep $TAG complete: $(ls -d $RUNS/${TAG}_* 2>/dev/null | grep -v '\.log' | wc -l) run dirs"
