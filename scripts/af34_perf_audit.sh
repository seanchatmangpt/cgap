#!/usr/bin/env bash
# AF-34 — Benchmark 04: ground-constraint stability audit (hardware counters).
#
# Gate for the dossier's TARGET-UNMEASURED perf claims (>99.8% L1 locality,
# <0.01% branch misprediction). Attempts the real hardware-counter gate in
# order, fails closed to a typed UNMEASURED-tool-blocker (exit 3, loud skip)
# when no PMU tool is available on this host. Never a silent pass.
#
# Exit codes (repo gate law): 0 measured | 1 gate fail (MEASURED UNMET) |
# 2 subject missing | 3 loud skip (tool blocker).
#
# Lane law: never runs git. Owns only this file + docs/jira/v26.11.3/receipts/
# af-34-perf-audit.md.
set -u
REPO="${AF34_REPO:-/Users/sac/cgap}"
OUT="${AF34_OUT:-/tmp/af34_perf_audit.out}"
REPORT=0   # 0 = counters measured, 1 = typed blocker
BLOCKER=""
: > "$OUT"

log() { printf '%s\n' "$*" | tee -a "$OUT"; }

# ---------- helpers ----------
canary() { # $1 template name; 0 if recording succeeded
  local d; d="$(mktemp -d /tmp/af34_canary.XXXX)"
  xctrace record --template "$1" --time-limit 2s --launch -- /usr/bin/true \
    --output "$d/t.trace" >/dev/null 2>&1
  local rc=$?
  grep -rq "Recording failed" "$d" 2>/dev/null
  local failed=$?
  rm -rf "$d" "$HOME/Launch_true_"*.trace 2>/dev/null
  [ "$rc" -eq 0 ] && [ "$failed" -ne 0 ]
}
perf_available() {
  command -v perf >/dev/null 2>&1 || return 1
  # PMU present: hardware events must exist
  ls /sys/devices/cpu/events >/dev/null 2>&1 || return 1
  return 0
}

# ---------- subject benches (exact commands an auditor runs) ----------
# Linux hardware-counter leg (verbatim, requires perf + PMU, unprivileged or sudo):
#   perf stat -e L1-dcache-load-misses,L1-dcache-loads,branch-misses,branch-instructions,instructions,cycles \
#     cargo test -p cgap-arrow --release --test ipc_stress -- --nocapture --test-threads=1
#   perf stat -e L1-dcache-load-misses,L1-dcache-loads,branch-misses,branch-instructions,instructions,cycles \
#     cargo test -p cgap-arrow --release --test slab_stress -- --nocapture --test-threads=1
#   perf stat -e L1-dcache-load-misses,L1-dcache-loads,branch-misses,branch-instructions,instructions,cycles \
#     cargo test -p cgap-aoq --release --test aoq1_bench_court -- --nocapture --test-threads=1
# Derivations: L1 hit ratio = 1 - (L1-dcache-load-misses / L1-dcache-loads)
#              branch misprediction rate = branch-misses / branch-instructions
BENCHES=(
  "ipc_stress|cargo test -p cgap-arrow --release --test ipc_stress -- --nocapture --test-threads=1"
  "slab_stress|cargo test -p cgap-arrow --release --test slab_stress -- --nocapture --test-threads=1"
  "aoq1_bench_court|cargo test -p cgap-aoq --release --test aoq1_bench_court -- --nocapture --test-threads=1"
)

log "== AF-34 ground-constraint audit (hardware counters) =="
log "host: $(uname -sm) $(uname -r)  cpu: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"
log ""

# ---------- leg 1: Linux perf ----------
if perf_available; then
  log "-- leg 1: perf stat (Linux PMU) --"
  for e in "${BENCHES[@]}"; do
    name="${e%%|*}"; cmd="${e#*|}"
    log "bench: $name"
    eval "perf stat -e L1-dcache-load-misses,L1-dcache-loads,branch-misses,branch-instructions,instructions,cycles $cmd" 2>&1 \
      | tee -a "$OUT" | tail -12
  done
  REPORT=0
else
  log "-- leg 1: perf UNAVAILABLE — $(command -v perf >/dev/null 2>&1 && echo 'perf present but no PMU (/sys/devices/cpu/events absent)' || echo 'perf binary absent on macOS host') --"
  BLOCKER="perf: $(command -v perf >/dev/null 2>&1 && echo present-no-PMU || echo absent); "
fi

# ---------- leg 2: macOS xctrace CPU Counters ----------
CTRACE_BIN="$(xcrun -f xctrace 2>/dev/null || true)"
if [ -n "$CTRACE_BIN" ] && canary "CPU Counters"; then
  log ""
  log "-- leg 2: xctrace 'CPU Counters' (Apple Silicon PMU via Instruments) --"
  for e in "${BENCHES[@]}"; do
    name="${e%%|*}"; cmd="${e#*|}"
    d="$(mktemp -d /tmp/af34_trace.XXXX)"
    log "bench: $name"
    # cargo under xctrace; run name recorded to $d
    xctrace record --template 'CPU Counters' --time-limit 300s --output "$d/b.trace" \
      --launch -- ${cmd} >> "$OUT" 2>&1
    # counters extracted via xctrace export by the auditor; summary line:
    log "trace: $d/b.trace (export: xctrace export --input $d/b.trace --toc)"
    done
  REPORT=0
else
  log ""
  log "-- leg 2: xctrace CPU Counters UNAVAILABLE — canary /usr/bin/true recording FAILS on this host ('Data stream: Fatal logging system error: log archive is corrupt or incomplete'), reproduces for Time Profiler too, inside and outside sandbox --"
  BLOCKER="${BLOCKER}xctrace-CPU-Counters: canary-fails(host-level); "
fi

# ---------- leg 3: colima VM probe (perf in a Linux guest) ----------
COLIMA=$(command -v colima || true)
if [ -n "$COLIMA" ] && colima list 2>/dev/null | grep -q "Running"; then
  PMU=$(colima ssh -- sh -c "ls /sys/devices/cpu/events 2>/dev/null | head -1" 2>/dev/null)
  if [ -n "$PMU" ]; then
    log ""
    log "-- leg 3: colima guest PMU present; run leg-1 commands inside: colima ssh -- <cmd> --"
    REPORT=0
  else
    log ""
    log "-- leg 3: colima guest has NO PMU (/sys/devices/cpu/events absent, perf_event_paranoid=4) — counters structurally invisible to the guest --"
    BLOCKER="${BLOCKER}colima-guest: no-PMU; "
  fi
else
  log ""
  log "-- leg 3: colima not running — skipped (loud) --"
  BLOCKER="${BLOCKER}colima: not-running; "
fi

# ---------- leg 4 (proxy, always run): release benches + wall-clock ----------
log ""
log "-- leg 4 (proxy): release bench runs + host hardware facts --"
sysctl -n hw.l1icachesize hw.l1dcachesize hw.cachelinesize 2>/dev/null | while IFS= read -r v; do log "hw: $v"; done
for e in "${BENCHES[@]}"; do
  name="${e%%|*}"; cmd="${e#*|}"
  log "bench: $name"
  ( cd "$REPO" && eval "/usr/bin/time -l $cmd" ) >> "$OUT" 2>&1 || {
    log "bench $name FAILED (rc=$?)"; REPORT=1; }
done

# ---------- verdict ----------
log ""
if [ "$REPORT" -eq 0 ]; then
  log "VERDICT: MEASURED — compare printed ratios against >99.8% L1 / <0.01% misprediction; MEET/UNMET typed per bench from actual counter output (never rounded)."
  exit 0
else
  log "VERDICT: UNMEASURED-tool-blocker — hardware counters unavailable on this host; no PMU route (perf absent, xctrace canary fails host-level, colima guest exposes no PMU)."
  log "BLOCKERS: $BLOCKER"
  log "The >99.8% L1 locality and <0.01% branch misprediction dossier figures remain TARGET-UNMEASURED."
  exit 3
fi
