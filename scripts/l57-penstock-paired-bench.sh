#!/bin/zsh
# l57-penstock-paired-bench.sh — KD-2 residue: penstock row 7 contended-band
# re-run (lane L57). Extends L27's paired method (scripts/l27-kd2-paired-bench.sh)
# to the penstock stage leg (stage_penstock_admission, chains-ingress bench).
#
# Identical method: N paired rounds; each round runs the SAME bench twice —
# SOLO then CONTENDED (RED-10 W4 classes via scripts/l27-loadgen.c,
# `combined 6 9`) — only the environment differs. Per-profile result =
# min-of-N over round values for fastest; median/slowest reported from the
# min-round frame. Ratio fence = contended/solo on matched columns.
# Sleep-free: real threads, divan's own timer.
#
# KD-COVERS: KD-2
set -u
REPO=/Users/sac/cgap
ROUNDS=3
OUT=/tmp/l57-penstock-paired
mkdir -p "$OUT"
LOG="$OUT/results.tsv"
: > "$LOG"
echo -e "round\tprofile\tfastest_ms\tmedian_ms\tslowest_ms" >> "$LOG"

clang -O2 -o "$OUT/l27-loadgen" "$REPO/scripts/l27-loadgen.c" || exit 2

run_bench() {  # $1 tag
  (cd "$REPO/crates/bench" && CARGO_TARGET_DIR=/tmp/l57-penstock-target \
    cargo bench --bench chains-ingress -- stage_penstock_admission) > "$OUT/bench_$1.log" 2>&1
  echo "bench rc=$? ($1)" >> "$OUT/rc.log"
}

parse_last() {  # $1 log; $2 round; $3 profile. Same divan row shape as L27:
                # `╰─ 1048576  <fastest> <unit> │ <slowest> ... │ <median> ...`
  f=$1; r=$2; p=$3
  row=$(grep '1048576' "$f" | grep '╰─' | tail -1 | sed 's/│//g')
  fa=$(echo "$row" | awk '{print $3, $4}')
  sl=$(echo "$row" | awk '{print $5, $6}')
  me=$(echo "$row" | awk '{print $7, $8}')
  echo -e "$r\t$p\t$fa\t$me\t$sl" >> "$LOG"
}

for i in $(seq 1 $ROUNDS); do
  run_bench "solo_$i"
  parse_last "$OUT/bench_solo_$i.log" "$i" "solo"

  # CONTENDED leg: combined bw6 + cpu9 (same loadgen profile as L27).
  "$OUT/l27-loadgen" combined 6 9 & LOADPID=$!
  run_bench "cont_$i"
  kill -INT $LOADPID 2>/dev/null
  wait $LOADPID 2>/dev/null
  parse_last "$OUT/bench_cont_$i.log" "$i" "contended"
done

echo "=== l57 paired penstock results ==="
cat "$LOG"
exit 0
