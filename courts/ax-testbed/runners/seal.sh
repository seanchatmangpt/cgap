#!/bin/sh
# AX-8 seal runner: invoke the honesty-gated receipt sealer INSIDE the
# cgap-ax-testbed image (AX-0 LOCAL docker; podman absent).
#
# Usage:
#   seal.sh -in <input.json> -out <output.md> [--sandbox gvisor|runc]
#
# SL-004 where-it-ran fields are filled HERE from real observables:
#   image_digest      <- docker inspect of the exact image tag
#   ax_runner_version <- /usr/local/share/ax_commit.txt inside the container
#   sandbox_class     <- --sandbox arg (default runc; docker Desktop = runc)
set -eu
IN=""; OUT=""; SBX="runc"; IMAGE="${IMAGE:-cgap-ax-testbed:ax1}"
SEAL_DIR="${SEAL_DIR:-/Users/sac/cgap/courts/interop-chicago/cmd/seal_receipt}"
while [ $# -gt 0 ]; do
  case "$1" in
    -in) IN="$2"; shift 2;;
    -out) OUT="$2"; shift 2;;
    --sandbox) SBX="$2"; shift 2;;
    *) echo "unknown arg $1" >&2; exit 2;;
  esac
done
[ -n "$IN" ] && [ -n "$OUT" ] || { echo "usage: seal.sh -in in.json -out out.md" >&2; exit 2; }
DIGEST="$( { docker inspect --format '{{index .RepoDigests 0}}' "$IMAGE" 2>/dev/null \
  || docker inspect --format '{{.Id}}' "$IMAGE"; } | sed -n 's/.*\(sha256:[0-9a-f]\{64\}\).*/\1/p')"
IN_DIR="$(cd "$(dirname "$IN")" && pwd)"; IN_F="$(basename "$IN")"
OUT_DIR="$(cd "$(dirname "$OUT")" && pwd)"; OUT_F="$(basename "$OUT")"
mkdir -p "$OUT_DIR"
exec docker run --rm \
  --entrypoint /bin/sh \
  -e AX_IMAGE_DIGEST="$DIGEST" \
  -e AX_SANDBOX_CLASS="$SBX" \
  -v "$SEAL_DIR":/work/sealer:ro \
  -v "$IN_DIR":/work/in:ro \
  -v "$OUT_DIR":/work/out \
  "$IMAGE" /work/sealer/container_run.sh \
    "/work/in/$IN_F" "/work/out/$OUT_F"
