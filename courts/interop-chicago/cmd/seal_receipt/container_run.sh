#!/bin/sh
# AX-8 container-side sealer invocation path (runs INSIDE cgap-ax-testbed).
# Boot env (mirrors entrypoint.sh, minus the ax-task-runner exec), build the
# sealer from the mounted source, then run it against the mounted fixture.
set -eu
export GOROOT="${GOROOT:-/usr/local/go}"
export CARGO_HOME="${CARGO_HOME:-/usr/local/cargo}"
export PATH="${GOROOT}/bin:${CARGO_HOME}/bin:${PATH}"
export TMPDIR="${TMPDIR:-/tmp}"; mkdir -p "$TMPDIR"
RUNNER_VER="ax-task-runner@$(cat /usr/local/share/ax_commit.txt 2>/dev/null || echo unknown)"
cd /work/sealer
go vet main.go
go build -o /work/seal_receipt main.go
exec /work/seal_receipt -fill-where \
  -ax-image-digest "$AX_IMAGE_DIGEST" \
  -ax-runner-version "$RUNNER_VER" \
  -ax-sandbox-class "$AX_SANDBOX_CLASS" \
  -in "$1" -out "$2"
