#!/bin/sh
# AX-1 entrypoint: env boot + once-only boot telemetry, then exec the real
# stage-1 ax-task-runner binary built at the pinned commit.
set -eu

export GOROOT="${GOROOT:-/usr/local/go}"
export CARGO_HOME="${CARGO_HOME:-/usr/local/cargo}"
export PATH="${GOROOT}/bin:${CARGO_HOME}/bin:${PATH}"
export CGO_ENABLED=1

# TMPDIR must exist inside the container (no_std courts and rustup helpers
# both assume a writable temp dir).
mkdir -p "${TMPDIR:-/tmp}" && export TMPDIR="${TMPDIR:-/tmp}"

# Boot telemetry, emitted exactly once (marker file guards repeats).
TELEMETRY_MARKER="/var/local/.ax_boot_telemetry_done"
if [ ! -f "${TELEMETRY_MARKER}" ]; then
  printf '{"event":"ax_testbed_boot","image":"cgap-ax-testbed","ax_commit":"%s","ts":"%s"}\n' \
    "$(cat /usr/local/share/ax_commit.txt 2>/dev/null || echo unknown)" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /dev/stderr
  : > "${TELEMETRY_MARKER}"
fi

# Exec the REAL runner binary. A missing binary is a loud fail, never a
# placeholder shell.
if [ ! -x /usr/local/bin/ax-task-runner ]; then
  echo "COURT FAIL: ax-task-runner binary missing from image" >&2
  exit 1
fi
exec /usr/local/bin/ax-task-runner "$@"
