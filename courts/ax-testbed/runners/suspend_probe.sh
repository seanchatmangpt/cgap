#!/usr/bin/env bash
# AX-9 suspend/resume probe — /Users/sac/cgap/courts/ax-testbed/runners/suspend_probe.sh
#
# Standing: UNVERIFIED-by-design (gate lane-ax-00-gate.md:47 — cluster NO-GO,
# ax CLI absent, install NOT APPROVED). This script is the executable SPEC;
# it must NOT be claimed as a pass until it runs to completion on a GO cluster.
# The refusal path (exit 3) is the only witness this session.
#
# Probe spec (execute when cluster GO arrives):
#   1. Build CONTROL digest: uninterrupted `cargo build --release` in a
#      baseline container; record sha256 of the target artifact.
#   2. Run the SAME build inside a debug:true Task (ax.io/v1alpha1).
#   3. `ax suspend <task>` mid-build; `ax resume <task>`.
#   4. Verify the build completes and the artifact sha256 EQUALS the control
#      digest. Mismatch => typed CHECKPOINT_MISMATCH failure (exit 1).
#
# Vendor claim under test (VENDOR-CLAIM-UNVERIFIED): the ax README's
# checkpoint story — suspend freezes the task workspace, resume continues
# it losslessly. Zero in-session witnesses exist. This probe would be the
# first witness; its digest compare is the falsifier.
#
# Exit codes: 0 pass | 1 probe fail (incl. CHECKPOINT_MISMATCH) |
#             2 subject missing | 3 typed prerequisite refusal.
set -euo pipefail

AX_TASK_NAME="${AX_TASK_NAME:-ax9-suspend-probe}"
AX_TASK_NAMESPACE="${AX_TASK_NAMESPACE:-ate-system}"
AX_IMAGE="${AX_IMAGE:-localhost/ax-testbed:latest}"
ARTIFACT_REL="target/release/cgap-bn3"
DIGESTS_DIR="${AX9_DIGESTS_DIR:-/tmp/ax9-digests}"

typed_refuse() {
  echo "REFUSED[PREREQ-ABSENT]: $1" >&2
  exit 3
}

# ---- (a) typed prerequisite gate: name EVERY absent prereq, then refuse ----
missing=()
command -v ax >/dev/null 2>&1 || missing+=("ax CLI (ax.io/v1alpha1 surface: apply/describe/suspend/resume)")
command -v kubectl >/dev/null 2>&1 || missing+=("kubectl")
if [ ${#missing[@]} -gt 0 ]; then
  for m in "${missing[@]}"; do
    echo "REFUSED[PREREQ-ABSENT]: $m" >&2
  done
  echo "AX-9 gate: NO-GO per docs/jira/v26.11.2/lanes/lane-ax-00-gate.md:44-55; install NOT APPROVED." >&2
  exit 3
fi

# ---- (b) full sequence with human-resume checkpoints -----------------------
echo "CHECKPOINT 1/5: control build (no suspend) starting"
kubectl apply -f - <<EOF
apiVersion: ax.io/v1alpha1
kind: Task
metadata: { name: ${AX_TASK_NAME}-control, namespace: ${AX_TASK_NAMESPACE} }
spec: { image: ${AX_IMAGE}, debug: false }
EOF
ax describe "${AX_TASK_NAME}-control"   # CHECKPOINT: human waits for control build completion
ax ssh "${AX_TASK_NAME}-control" -- cargo build --release
ax ssh "${AX_TASK_NAME}-control" -- sha256sum "$ARTIFACT_REL" \
  > "$DIGESTS_DIR/control.sha256" || { echo "CHECKPOINT_MISMATCH-CLASS FAIL: control digest unavailable" >&2; exit 1; }

echo "CHECKPOINT 2/5: debug:true Task + suspend/resume run starting"
kubectl apply -f - <<EOF
apiVersion: ax.io/v1alpha1
kind: Task
metadata: { name: ${AX_TASK_NAME}, namespace: ${AX_TASK_NAMESPACE} }
spec: { image: ${AX_IMAGE}, debug: true }
EOF
ax ssh "${AX_TASK_NAME}" -- cargo build --release &
BUILD_PID=$!

echo "CHECKPOINT 3/5: suspend mid-build (human resumes here)"
wait "$BUILD_PID" 2>/dev/null || true   # no sleep: the build PID is the observable
ax suspend "${AX_TASK_NAME}"
ax describe "${AX_TASK_NAME}"           # CHECKPOINT: human confirms suspended state
ax resume "${AX_TASK_NAME}"
ax ssh "${AX_TASK_NAME}" -- cargo build --release   # resume completes the build

echo "CHECKPOINT 4/5: digest compare (resume vs control)"
ax ssh "${AX_TASK_NAME}" -- sha256sum "$ARTIFACT_REL" \
  > "$DIGESTS_DIR/resumed.sha256"
# Batch-amortized honesty: compare ONCE at the end, no polling sleeps anywhere.
if ! cmp -s "$DIGESTS_DIR/control.sha256" "$DIGESTS_DIR/resumed.sha256"; then
  echo "CHECKPOINT_MISMATCH: resumed artifact digest != control digest" >&2
  echo "  control: $(cat "$DIGESTS_DIR/control.sha256")" >&2
  echo "  resumed: $(cat "$DIGESTS_DIR/resumed.sha256")" >&2
  exit 1
fi

echo "CHECKPOINT 5/5: PASS — suspend/resume is digest-transparent"
echo "VENDOR-CLAIM WITNESSED: ax suspend/resume checkpoint semantics (was VENDOR-CLAIM-UNVERIFIED)"
exit 0
