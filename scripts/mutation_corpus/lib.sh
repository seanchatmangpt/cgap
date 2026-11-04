#!/usr/bin/env bash
# mutation_corpus/lib.sh — anti-vacuity mutation harness (lane-2, v26.11.4).
#
# Generalizes smalt's scripts/shim_court/lib.sh protocol to sandboxed
# subjects: every mutation lands in a /tmp sandbox staged from
# `git archive HEAD` — the canonical checkout is NEVER edited in place.
#
#   1. STAGE      sandbox from git archive HEAD (clean HEAD baseline, no
#                 in-flight lane edits ride along).
#   2. BASELINE   sha256 the subject file in the sandbox.
#   3. BEFORE     run the gate against the PRISTINE sandbox; exit 0 = the
#                 old (lawful) signature holds. Nonzero = the corpus leg
#                 itself is broken; refuse to shim.
#   4. SHIM       caller-defined apply_mutation hook edits the SANDBOX
#                 subject in place, marked "SHIM-COURT".
#   5. AFTER      run the gate against the MUTATED sandbox; nonzero exit =
#                 the gate FLIPPED. Exit 0 here = court vacuous -> FAIL.
#   6. RESTORE    restore from backup; sha256 must equal baseline byte-exact,
#                 fail-closed. Receipt JSON records every phase.
#
# Exit 0 only if all phases pass. Typed FAIL lines per phase:
#   [FAIL:STAGE] [FAIL:BASELINE] [FAIL:BEFORE] [FAIL:SHIM] [FAIL:AFTER] [FAIL:RESTORE]
# A leg needing a missing environment must SKIP LOUDLY via mc_skip (exit 3,
# never a faked pass).
#
# Receipt: <repo>/scripts/mutation_corpus/receipts/<name>.json
#   {name, court, flipped, restore_verified, baseline_sha, restore_sha,
#    before_exit, after_exit, sandbox, plane, timestamp}

set -euo pipefail

MC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CGAP_ROOT="$(cd "$MC_DIR/../.." && pwd)"
MC_RECEIPTS_DIR="$MC_DIR/receipts"
MC_SANDBOX="${MC_SANDBOX:-$(mktemp -d /tmp/cgap-mut-sandbox.XXXXXX)}"
MC_CARGO_SANDBOX="${MC_CARGO_SANDBOX:-$(mktemp -d /tmp/cgap-mut-cargo.XXXXXX)}"
MC_TARGET_DIR="${MC_TARGET_DIR:-$MC_CARGO_SANDBOX/target}"
MC_PLANE="cgap-local"
export MC_SANDBOX MC_CARGO_SANDBOX MC_TARGET_DIR MC_PLANE

mkdir -p "$MC_RECEIPTS_DIR"

mc_log() { printf '[%s] %s\n' "$1" "$2"; }

mc_fail() { # mc_fail <PHASE> <detail>
    printf '[FAIL:%s] %s\n' "$1" "$2" >&2
    exit 1
}

mc_skip() { # loud skip with reason; recorded as SKIP, never PASS
    printf '[SKIP] %s: %s\n' "${MC_NAME:-mutation}" "$1"
    exit 3
}

mc_sha() { shasum -a 256 "$1" 2>/dev/null | awk '{print $1}'; }

# Self-check: prove the sha256 restore rule DETECTS a corrupted restore
# (real temp-copy exercise, no mocks). Run once per corpus script.
mc_self_check() {
    local dir tmp backup base good corrupt
    dir="$(mktemp -d)"
    tmp="$dir/subject.txt"
    printf 'subject-body\n' > "$tmp"
    backup="$dir/backup"
    cp "$tmp" "$backup"
    base="$(mc_sha "$tmp")"
    printf 'SHIM-COURT body\n' > "$tmp"
    cp "$backup" "$tmp"
    good="$(mc_sha "$tmp")"
    cp "$backup" "$tmp"
    printf 'x' | dd of="$tmp" bs=1 seek=3 conv=notrunc status=none 2>/dev/null
    corrupt="$(mc_sha "$tmp")"
    rm -rf "$dir"
    [ "$good" = "$base" ] && [ "$corrupt" != "$base" ] || {
        printf '[FAIL:SELF-CHECK] restore-detection self-check failed\n' >&2
        return 1
    }
    mc_log SELF-CHECK "sha256 restore-detection verified"
    return 0
}

# mc_stage_full — full-repo sandbox from git archive HEAD (for census-class
# gates that walk the whole workspace).
mc_stage_full() {
    if [ -d "$MC_SANDBOX/.git" ] || [ ! -d "$MC_SANDBOX" ]; then
        mkdir -p "$MC_SANDBOX"
        git -C "$CGAP_ROOT" archive HEAD | tar -x -C "$MC_SANDBOX"
    fi
    [ -f "$MC_SANDBOX/Cargo.toml" ] || mc_fail STAGE "archive staging failed: $MC_SANDBOX"
    mc_log STAGE "full sandbox staged: $MC_SANDBOX (git archive HEAD)"
}

# mc_stage_cargo — minimal 3-crate cargo workspace (cgap-blake3 + cgap-crypto
# + cgap-aoq), manifests trimmed of dev-deps/bench targets the corpus courts
# do not need. From git archive HEAD: clean baseline, no lane in-flight edits.
mc_stage_cargo() {
    local sb="$MC_CARGO_SANDBOX" c
    mkdir -p "$sb/crates"
    cp "$CGAP_ROOT/LICENSE.md" "$sb/LICENSE.md"
    for c in cgap-blake3 cgap-crypto cgap-aoq; do
        mkdir -p "$sb/crates/$c"
        cp "$CGAP_ROOT/crates/$c/Cargo.toml" "$sb/crates/$c/Cargo.toml"
        cp -R "$CGAP_ROOT/crates/$c/src" "$sb/crates/$c/src"
        mkdir -p "$sb/crates/$c/tests"
    done
    cp "$CGAP_ROOT/crates/cgap-blake3/tests/zero_alloc.rs" \
       "$CGAP_ROOT/crates/cgap-blake3/tests/chain_mutation_court.rs" \
       "$sb/crates/cgap-blake3/tests/"
    cp "$CGAP_ROOT/crates/cgap-aoq/tests/aoq1_layout_drift_court.rs" \
       "$sb/crates/cgap-aoq/tests/"
    cat > "$sb/Cargo.toml" <<'TOML'
[workspace]
members = ["crates/cgap-blake3", "crates/cgap-crypto", "crates/cgap-aoq"]
resolver = "2"

[workspace.package]
version = "26.11.3"
edition = "2021"
rust-version = "1.85"
license-file = "LICENSE.md"
repository = "https://github.com/seanchatmangpt/cgap"
authors = ["Sean Chatman <info@chatmangpt.com>"]
TOML
    mc_fix_manifests "$sb"
    : > "$sb/staged"
    mc_log STAGE "cargo sandbox staged: $sb"
}

mc_fix_manifests() {
    local sb="$1"
    python3 - "$sb" <<'PYEOF'
import sys
sb = sys.argv[1]
# workspace license-file must resolve from the sandbox root
# blake3: drop dev-deps (zero_alloc/chain courts need only the lib)
t = open(f"{sb}/crates/cgap-blake3/Cargo.toml").read()
t = t.split("[dev-dependencies]")[0]
open(f"{sb}/crates/cgap-blake3/Cargo.toml", "w").write(t)
# crypto: drop dev-deps (divan/chacha external) and the bench target block
import re
t = open(f"{sb}/crates/cgap-crypto/Cargo.toml").read()
t = t.split("[dev-dependencies]")[0]
t = re.sub(r"\[\[bench\]\][^\[]*", "", t)
open(f"{sb}/crates/cgap-crypto/Cargo.toml", "w").write(t)
# aoq: keep lib dep (cgap-crypto); trim dev-deps + explicit [[test]] block
t = open(f"{sb}/crates/cgap-aoq/Cargo.toml").read()
t = t.split("[dev-dependencies]")[0]
open(f"{sb}/crates/cgap-aoq/Cargo.toml", "w").write(t)
PYEOF
}

# mc_with_mutation <name> <court-id> <subject-file> <before-cmd> <after-cmd>
# Requires caller-defined apply_mutation (edits $subject in place, marked
# SHIM-COURT). Writes receipt JSON; exit 0 iff BEFORE green + AFTER flips +
# restore byte-exact.
mc_with_mutation() {
    local name="$1" court="$2" subject="$3" before_cmd="$4" after_cmd="$5"
    MC_NAME="$name"
    local baseline before_rc after_rc restore_sha
    if [ ! -f "$subject" ]; then
        mc_fail BASELINE "subject file not found: $subject"
    fi
    baseline="$(mc_sha "$subject")"
    mc_log SHIM "baseline sha256($subject) = $baseline"

    SHIM_PHASE="BEFORE"
    if ! bash -uc "$before_cmd"; then
        mc_fail BEFORE "before-witness failed on PRISTINE subject — corpus leg broken; refusing to shim ($before_cmd)"
    fi
    mc_log SHIM "BEFORE witness passed (gate green on pristine subject)"

    SHIM_PHASE="SHIM"
    SHIM_BACKUP="$(mktemp "${TMPDIR:-/tmp}/mc.orig.XXXXXX")"
    cp "$subject" "$SHIM_BACKUP"
    if ! apply_mutation "$subject"; then
        cp "$SHIM_BACKUP" "$subject"
        mc_fail SHIM "apply_mutation failed; subject restored from backup"
    fi
    if ! grep -q 'SHIM-COURT' "$subject"; then
        cp "$SHIM_BACKUP" "$subject"
        mc_fail SHIM "mutated subject carries no SHIM-COURT marker; refusing to continue"
    fi
    mc_log SHIM "mutation applied to $subject (SHIM-COURT marker present)"

    SHIM_PHASE="AFTER"
    set +e
    bash -uc "$after_cmd"
    after_rc=$?
    set -e
    if [ "$after_rc" -eq 0 ]; then
        cp "$SHIM_BACKUP" "$subject"
        restore_sha="$(mc_sha "$subject")"
        mc_write_receipt "$name" "$court" "$subject" false "$baseline" "$restore_sha" "$after_rc"
        [ "$restore_sha" = "$baseline" ] || mc_fail RESTORE "restore-after-vacuous mismatch"
        mc_fail AFTER "after-witness EXITED 0 on MUTATED subject — court vacuous (COURT FAILS); subject restored; vacuity receipt written"
    fi
    mc_log AFTER "AFTER witness: gate flipped (exit $after_rc, nonzero as required)"

    SHIM_PHASE="RESTORE"
    cp "$SHIM_BACKUP" "$subject"
    restore_sha="$(mc_sha "$subject")"
    rm -f "$SHIM_BACKUP"; SHIM_BACKUP=""
    if [ "$restore_sha" != "$baseline" ]; then
        mc_fail RESTORE "sha256 mismatch after restore: $restore_sha != baseline $baseline — subject CORRUPTED"
    fi
    mc_log SHIM "RESTORE verified byte-exact"

    MC_FLIPPED=true MC_BEFORE_RC=0 MC_AFTER_RC="$after_rc" \
    MC_BASELINE_SHA="$baseline" MC_RESTORE_SHA="$restore_sha" \
        mc_write_receipt "$name" "$court" "$subject" true "$baseline" "$restore_sha" "$after_rc"
    printf '[PASS] %s: non-vacuous (BEFORE green, AFTER flips gate, restore sha256-exact)\n' "$name"
    return 0
}

mc_write_receipt() {
    # mc_write_receipt <name> <court> <subject> <flipped-true|false> <baseline-sha> <restore-sha> <after-exit>
    local name="$1" court="$2" subject="$3"
    local flipped="${4:-true}" bsha="${5:-}" rsha="${6:-}" arc="${7:-0}"
    MC_NAME="$name"
    python3 - "$MC_RECEIPTS_DIR" "$name" "$court" "$subject" "$flipped" "$bsha" "$rsha" "$arc" <<'PYEOF'
import json, os, sys, datetime
rdir, name, court, subject, flipped, bsha, rsha, arc = sys.argv[1:9]
d = {
    "name": name,
    "court": court,
    "subject": subject,
    "flipped": flipped == "true",
    "court_vacuous": flipped != "true",
    "restore_verified": rsha == bsha and bool(rsha),
    "baseline_sha256": bsha,
    "restore_sha256": rsha,
    "after_exit": int(arc),
    "sandbox": os.environ.get("MC_SANDBOX", ""),
    "cargo_sandbox": os.environ.get("MC_CARGO_SANDBOX", ""),
    "plane": os.environ.get("MC_PLANE", "cgap-local"),
    "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
p = os.path.join(rdir, name + ".json")
with open(p, "w") as fh:
    json.dump(d, fh, indent=2)
    fh.write("\n")
print(f"[receipt] {p}")
PYEOF
}
