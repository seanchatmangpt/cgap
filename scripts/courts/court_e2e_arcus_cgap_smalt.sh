#!/usr/bin/env bash
# court_e2e_arcus_cgap_smalt.sh
# End-to-End Continuous Pipeline Qualification Court: arcus -> cgap -> smalt
# Cross-Platform: Autodetects Darwin / Linux tmpfs mounts & cache padding
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CGAP_ROOT="${WORKSPACE_ROOT}/cgap"
ARCUS_ROOT="${WORKSPACE_ROOT}/arcus"
SMALT_ROOT="${WORKSPACE_ROOT}/smalt"

# Detect OS-native tmpfs shared memory location
if [[ "$(uname -s)" == "Linux" ]] && [[ -d "/dev/shm" ]]; then
    SHM_PATH="/dev/shm/sovereign_e2e_arrow.arena"
else
    SHM_PATH="/tmp/sovereign_e2e_arrow.arena"
fi

RECEIPT_OUT="${WORKSPACE_ROOT}/cgap/reports/e2e_cross_subsystem_receipt.json"
FRAMES=1000000

# ------------------------------------------------------------------------------
# OI-4 admission-ratio band (docs/jira/v26.11.6/open-items.md)
# Observed: 355,824 / 1,000,000 admitted (35.58%); the SPSC membrane refuses at
# saturation (lawful per regime 3), so a refusal-heavy ratio is EXPECTED.
# Band is an empirical envelope, not a spec: [ADMIT_BAND_LO, ADMIT_BAND_HI].
# Below LO = membrane starved/over-refusing; above HI = saturation refusal path
# no longer exercised (court would be vacuous on regime 3).
# ------------------------------------------------------------------------------
ADMIT_BAND_LO="${ADMIT_BAND_LO:-0.20}"
ADMIT_BAND_HI="${ADMIT_BAND_HI:-0.60}"

band_check() { # $1 = receipt json path; exit 0 in band, 1 out of band
    python3 -c "
import json, sys
r = json.load(open(sys.argv[1]))
lo, hi = float(sys.argv[2]), float(sys.argv[3])
ratio = r['arcus_admitted'] / r['arcus_sent']
print(f'  Admission ratio: {ratio:.4f} (band [{lo}, {hi}])')
sys.exit(0 if lo <= ratio <= hi else 1)
" "$1" "${ADMIT_BAND_LO}" "${ADMIT_BAND_HI}"
}

if [[ "${1:-}" == "--band-selftest" ]]; then
    T="$(mktemp -d)"
    echo '{"arcus_sent":1000000,"arcus_admitted":355824}' > "${T}/ok.json"
    echo '{"arcus_sent":1000000,"arcus_admitted":1000000}' > "${T}/mut.json"
    band_check "${T}/ok.json" || { echo "[FAIL] observed ratio rejected"; exit 1; }
    if band_check "${T}/mut.json"; then echo "[FAIL] mutant (100% admitted) passed band: VACUOUS"; exit 1; fi
    echo "[BAND SELFTEST]: observed in band, mutant (no refusals) flipped."
    rm -rf "${T}"
    exit 0
fi

# ------------------------------------------------------------------------------
# G7 preflight: verify every artifact BEFORE starting anything (gate contract:
# exit 2 = subject missing). Nothing is spawned or created until this passes.
# ------------------------------------------------------------------------------
MISSING=()
[[ -x "${CGAP_ROOT}/target/debug/cgap_e2e_consumer" ]] || MISSING+=("cgap consumer: ${CGAP_ROOT}/target/debug/cgap_e2e_consumer")
[[ -x "${ARCUS_ROOT}/bin/arcus_e2e_producer" ]] || MISSING+=("arcus producer: ${ARCUS_ROOT}/bin/arcus_e2e_producer")
[[ -d "${SMALT_ROOT}" ]] || MISSING+=("smalt dir: ${SMALT_ROOT}")
command -v mix > /dev/null 2>&1 || MISSING+=("mix (elixir) not on PATH")
if (( ${#MISSING[@]} > 0 )); then
    echo "[PREFLIGHT] subject missing (exit 2); nothing started:" >&2
    for m in "${MISSING[@]}"; do echo "  - ${m}" >&2; done
    exit 2
fi

CGAP_PID=""
cleanup() {
    local rc=$?
    trap - EXIT INT TERM
    if [[ -n "${CGAP_PID}" ]]; then kill "${CGAP_PID}" 2> /dev/null || true; wait "${CGAP_PID}" 2> /dev/null || true; fi
    rm -f "${SHM_PATH}"
    exit "${rc}"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "================================================================================"
echo "STARTING CROSS-PLATFORM E2E INTEGRATION COURT: arcus -> cgap -> smalt"
echo "Target Architecture: $(uname -s) $(uname -m)"
echo "Shared Memory Arena: ${SHM_PATH}"
echo "Mode: Continuous Zero-Copy Arrow Ingress & Clearing (${FRAMES} frames)"
echo "================================================================================"

# ------------------------------------------------------------------------------
# PASS 1: Nominal Pipeline Run
# ------------------------------------------------------------------------------
echo "--- PASS 1: NOMINAL CONTINUOUS INGRESS & CLEARING ---"
rm -f "${SHM_PATH}"
python3 -c "
import os
fd = os.open('${SHM_PATH}', os.O_CREAT | os.O_RDWR, 0o666)
os.ftruncate(fd, 64 * 1024 * 1024)
os.close(fd)
"

echo "[1/3] Starting cgap (Rust) SIMD consumer..."
export ARENA_PATH="${SHM_PATH}"
export TARGET_FRAMES="${FRAMES}"
"${CGAP_ROOT}/target/debug/cgap_e2e_consumer" &
CGAP_PID=$!

echo "[2/3] Injecting ${FRAMES} frames via arcus (Go) SPSC membrane..."
"${ARCUS_ROOT}/bin/arcus_e2e_producer" --arena "${SHM_PATH}" --total-frames "${FRAMES}"

wait ${CGAP_PID}
CGAP_PID=""

echo "[3/3] Finalizing state clearing via smalt (BEAM) clearinghouse..."
cd "${SMALT_ROOT}"
mix run --no-start -e 'Smalt.Clearinghouse.emit_receipt("'"${SHM_PATH}"'", "'"${RECEIPT_OUT}"'")'

echo "Verifying nominal receipt conservation..."
python3 -c "
import json
with open('${RECEIPT_OUT}') as f:
    receipt = json.load(f)

sent = receipt['arcus_sent']
admitted = receipt['arcus_admitted']
refused = receipt['arcus_refused']
pruned = receipt['cgap_bn3_pruned']
cleared = receipt['smalt_state_cleared']
blake3_root = receipt['blake3_root']

print(f'  Total Sent:     {sent:,}')
print(f'  arcus Refused:  {refused:,}')
print(f'  cgap BN3 Prune: {pruned:,}')
print(f'  smalt Cleared:  {cleared:,}')
print(f'  Attestation:    {blake3_root}')

assert sent == (admitted + refused), f'Ingress conservation broken: {sent} != {admitted} + {refused}'
assert admitted == (pruned + cleared), f'Processing conservation broken: {admitted} != {pruned} + {cleared}'
assert admitted > 0, 'Zero frames admitted'
assert cleared > 0, 'Zero frames cleared'
print('[PASS 1 WITNESS]: Nominal conservation verified.')
"
band_check "${RECEIPT_OUT}" || { echo "[ERROR]: admission ratio outside band"; exit 1; }
echo "[PASS 1 WITNESS]: admission ratio within band."

# ------------------------------------------------------------------------------
# PASS 2: Adversarial Anti-Vacuity Control (Tamper Detection)
# ------------------------------------------------------------------------------
echo ""
echo "--- PASS 2: ADVERSARIAL ANTI-VACUITY CONTROL (MUTATION INJECTION) ---"
echo "Injecting intentional corruption into shared memory header (corrupting admitted counter)..."
python3 -c "
import struct
with open('${SHM_PATH}', 'r+b') as f:
    f.seek(264) # Offset of TotalCleared
    f.write(struct.pack('<Q', 999999999)) # Corrupt count to break Admitted == Pruned + Cleared
"

echo "Executing smalt clearinghouse against corrupted arena (expecting fail-closed raise)..."
set +e
cd "${SMALT_ROOT}"
mix run --no-start -e 'Smalt.Clearinghouse.emit_receipt("'"${SHM_PATH}"'", "'"${RECEIPT_OUT}"'.mutant")' 2> /tmp/smalt_mutant.err
SMALT_EXIT=$?
set -e

if [ ${SMALT_EXIT} -ne 0 ]; then
    echo "[PASS 2 WITNESS]: Smalt actively detected corruption and aborted with exit code ${SMALT_EXIT}."
    echo "  Captured error signature:"
    tail -n 3 /tmp/smalt_mutant.err
else
    echo "[ERROR]: Smalt accepted corrupted arena! Anti-vacuity court failed!"
    exit 1
fi

rm -f "${SHM_PATH}" /tmp/smalt_mutant.err "${RECEIPT_OUT}.mutant"
echo "================================================================================"
echo "CROSS-PLATFORM E2E INTEGRATION COURT STATUS: ALIVE & CONFORMANT"
echo "================================================================================"
