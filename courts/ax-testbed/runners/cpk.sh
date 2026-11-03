#!/usr/bin/env bash
# LANE DOD-14 — membrane capability study (Cpk) driver for IPC-7.
#
# Measures the ADMISSION MEMBRANE layer (go cgo -> cgap-ffi AdmitFrame,
# batch-1365 amortized ns/frame) N=30 sequential trials, computes
# mean / sigma / Cpu, and applies the DECIDED membrane budget USL
# (250 ns/frame) as the gate. The 10 ns USL is kernel-layer-only
# (AX-GATE-03 layer law, dflss/10-ax-control-plan.md:27) — its Cpk is
# printed loudly (it will be negative) and NEVER gated on.
#
# Layer law: the membrane measures ~179-192 ns/frame amortized
# (IPC-3 W4 / AX-5 W4 darwin column; re-witnessed this session), NOT 10 ns.
#
# Exit contract (fail-closed, mirrors crates/bench/src/bin/cpk_study.rs):
#   0  decided-gate Cpu >= 1.33 AND flip witnessed (jitter -> Cpu < 1.33)
#   1  decided-gate Cpu < 1.33 — honest FAIL, correct output (G09 law)
#   2  VACUOUS: jitter flip did NOT drive Cpu below 1.33
#   3  loud skip: dylib / fixtures / go missing — names the rebuild command
set -u

REPO="${CGAP_REPO:-/Users/sac/cgap}"
WORK="${CGAP_DOD14_WORK:-/tmp/dod14-work}"
COURTS="$REPO/courts/interop-chicago"
DYLIB_DIR=/tmp/ipc3-target/release   # binding's baked darwin -L/-rpath
FX=/tmp/ipc3-fixtures
N=30
USL_DECIDED=250.0   # ns/frame, decided membrane budget (MV-5/R1 pattern)
USL_ASP=10.0        # ns/frame, kernel-layer-only aspirational; NEVER gated
CPK_FLOOR=1.33      # exit floor, mirrors cpk_study.rs

log() { printf '[DOD14] %s\n' "$*"; }

# --- thermals ---------------------------------------------------------------
THERM_BEFORE="$(pmset -g therm 2>/dev/null || echo 'pmset unavailable')"
log "thermals before: $THERM_BEFORE"

# --- loud-skip guards -------------------------------------------------------
command -v go >/dev/null 2>&1 || { log '[SKIP] go absent'; exit 3; }
[ -f "$DYLIB_DIR/libcgap_ffi.dylib" ] || {
  log '[SKIP] darwin dylib missing — rebuild with: CARGO_TARGET_DIR=/tmp/ipc3-target cargo build --release -p cgap-ffi'
  exit 3
}
[ -f "$FX/batch_1365.bin" ] || {
  log '[SKIP] fixtures missing — regen per lane-ipc-03 W1 (/tmp/ipc3-fxgen lease)'
  exit 3
}

# --- Phase C: N=30 darwin membrane trials (bounded, no sleeps) --------------
log "phase C: go bench BenchmarkAdmitBatch1365 -benchtime 200x -count $N (darwin membrane)"
mkdir -p "$WORK"
(cd "$COURTS" && CGAP_FIXTURE_DIR="$FX" go test -run XXX \
   -bench BenchmarkAdmitBatch1365 -benchtime 200x -count "$N" . ) \
   > "$WORK/bench30.txt" 2> "$WORK/bench30.stderr"
BENCH_RC=$?
[ "$BENCH_RC" -eq 0 ] || { log "bench FAILED rc=$BENCH_RC — see $WORK/bench30.stderr"; exit 1; }


# --- Phase C2: container context column (docker LOCAL per AX-0 W4) ---------
# CONTEXT ONLY, never gated (AX-5 W5 isolation-cost honesty). The courts
# module is copied into the work lease and the binding's linux LDFLAGS path
# (-L/work/target/release) is satisfied by the in-container cdylib build.
RT=""
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  RT=docker
fi
if [ -n "$RT" ] && [ -d /tmp/ipc3-fxgen ]; then
  mkdir -p "$WORK/ctx" "$WORK/src"
  cp -R "$COURTS" "$WORK/src/interop"
  cp -R /tmp/ipc3-fxgen "$WORK/ctx/fxgen"
  find "$WORK/ctx/fxgen" -name Cargo.toml -exec sed -i '' \
    -e "s|/Users/sac/cgap/crates|/repo/crates|g" {} + 2>/dev/null || true
  if "$RT" run --rm -v "$REPO":/repo:ro -v "$WORK":/work \
       -e CARGO_TARGET_DIR=/work/ctx/target \
       rust:1-bookworm bash -c 'cd /repo && cargo build --release --locked -p cgap-ffi' \
    && "$RT" run --rm -v "$WORK":/work -e CARGO_TARGET_DIR=/work/ctx/fxtarget \
       rust:1-bookworm bash -c 'cd /work/ctx/fxgen && cargo run --release -- /work/ctx/fixtures' \
    && "$RT" run --rm -v "$WORK":/work -e CGAP_FIXTURE_DIR=/work/ctx/fixtures \
       golang:1.23-bookworm bash -c \
       'cd /work/src/interop && go test -run XXX -bench BenchmarkAdmitBatch1365 -benchtime 200x -count 30 .' \
       > "$WORK/bench30_linux.txt" 2> "$WORK/bench30_linux.stderr"; then
    log "container context column captured -> $WORK/bench30_linux.txt"
  else
    log '[SKIP] container context arm failed — context-gap recorded, not a gate'
  fi
else
  log '[SKIP] docker daemon or fxgen lease absent — container context column skipped (loud)'
fi


# --- Phase D: stats + flip + exit contract (python3, fail-closed) -----------
log "phase D: capability stats (decided USL ${USL_DECIDED} ns/frame; aspirational ${USL_ASP} ns printed loudly, never gated)"
python3 - "$WORK/bench30.txt" "$WORK/bench30_linux.txt" "$N" "$USL_DECIDED" "$USL_ASP" <<'PY'
import math, re, sys, random, json
bench_path, linux_path, N = sys.argv[1], sys.argv[2], int(sys.argv[3])
USL, USL_ASP = float(sys.argv[4]), float(sys.argv[5])
FLOOR = 1.33
BATCH = 1365.0

def parse(path):
    try:
        lines = open(path).read().splitlines()
    except FileNotFoundError:
        return None
    out = []
    for line in lines:
        m = re.match(r'BenchmarkAdmitBatch1365\S*\s+\d+\s+([\d.]+) ns/op', line)
        if m:
            out.append(float(m.group(1)) / BATCH)  # ns/frame
    return out or None

def cap(samples, usl):
    n = len(samples)
    mean = sum(samples)/n
    sd = math.sqrt(sum((x-mean)**2 for x in samples)/(n-1))
    cpu = (usl - mean)/(3*sd)
    return mean, sd, cpu

s = parse(bench_path)
if s is None or len(s) != N:
    print(f"DOD14: parsed {0 if s is None else len(s)} darwin trials, expected {N}", file=sys.stderr)
    sys.exit(1)
mean, sd, cpu = cap(s, USL)
cpu_asp = cap(s, USL_ASP)[2]

# Anti-vacuity jitter flip: inject additive scheduler jitter
# (+U(0,60) ns on every sample, +U(0,120) extra on every 3rd) — a plausible
# scheduler-jitter model — and REQUIRE Cpu to drop below the floor.
rng = random.Random(0xA20C0FFEE5A7E000)
jf = [x + rng.uniform(0, 60.0) + (rng.uniform(0, 120.0) if i % 3 == 0 else 0.0) for i, x in enumerate(s)]
jmean, jsd, jcpu = cap(jf, USL)

linux_summary = "absent"
sl = parse(linux_path)
if sl:
    lm, lsd, lcpu = cap(sl, USL)
    linux_summary = {"n": len(sl), "mean": round(lm,2), "sigma": round(lsd,2),
                     "cpu_decided": round(lcpu,3), "cpu_aspirational": round(cap(sl, USL_ASP)[2],3)}

report = {
  "report": "dod14-membrane-cpk",
  "layer": "membrane (go cgo -> cgap-ffi AdmitFrame, batch-1365 amortized ns/frame)",
  "n": N,
  "usl_decided_ns_per_frame": USL,
  "usl_decided_provenance": "AX-GATE-03 re-scope band (IPC-3 W4 / AX-5 W4 darwin 179-192 ns/frame) + headroom; MV-5/R1 pattern",
  "usl_aspirational_ns_per_frame": USL_ASP,
  "cpk_exit_floor": FLOOR,
  "mean_ns_per_frame": round(mean,3),
  "sigma_ns_per_frame": round(sd,3),
  "min": round(min(s),2), "max": round(max(s),2),
  "cpu_decided": round(cpu,3),
  "cpu_aspirational_10ns": round(cpu_asp,3),
  "aspirational_note": "10 ns USL is kernel-layer-only (AX-GATE-03 layer law); Cpu negative, printed loudly, NEVER gated",
  "flip": {"model": "+U(0,60)ns every sample, +U(0,120)ns every 3rd, seed 0xA20C0FFEE5A7E000",
           "mean": round(jmean,3), "sigma": round(jsd,3), "cpu": round(jcpu,3),
           "dropped_below_floor": jcpu < FLOOR},
  "linux_container_context": linux_summary,
  "samples_ns_per_frame": [round(x,2) for x in s],
}
print(json.dumps(report, indent=1))
rc = 0
if cpu < FLOOR:
    rc = 1
if jcpu >= FLOOR:
    rc = 2
sys.exit(rc)
PY
DRC=$?
case "$DRC" in
  0) log "PASS: decided-gate Cpu >= $CPK_FLOOR and jitter flip witnessed (Cpu dropped below floor)" ;;
  1) log "FAIL: decided-gate Cpu < $CPK_FLOOR — honest FAIL recorded (G09 law)" ;;
  2) log "VACUOUS: jitter flip did NOT drive Cpu below $CPK_FLOOR — court FAIL" ;;
  *) log "stats phase error rc=$DRC" ;;
esac

THERM_AFTER="$(pmset -g therm 2>/dev/null || echo 'pmset unavailable')"
log "thermals after: $THERM_AFTER"
exit "$DRC"
