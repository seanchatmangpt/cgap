#!/bin/bash
# flip-tests.sh — anti-vacuity courts for the three SM-4 hooks.
# Each hook: one fixture that FIRES (observable effect) + one that DOES NOT.
# Exit 0 = all courts pass (both directions flip); nonzero = court FAIL.
set -u
H="/Users/sac/cgap/.claude/hooks"
chmod +x "$H"/*.sh 2>/dev/null
fail=0
J='{"tool_name":"Bash","tool_input":{"command":"%s"}}'

court() { # name expected_exit expected_effect cmd
  local name="$1" exp="$2" eff="$3" cmd="$4" out rc
  out="$(printf "$J" "$cmd" | "$H/$1.sh" 2>&1)"; rc=$?
  if [ "$rc" != "$exp" ]; then echo "[FAIL] $name exit=$rc want=$exp :: $out"; fail=1; return; fi
  case "$eff" in
    fire)   [ -n "$out" ] || { echo "[FAIL] $name expected firing output, got none"; fail=1; return; } ;;
    quiet)  [ -z "$out" ] || { echo "[FAIL] $name expected silence, got: $out"; fail=1; return; } ;;
  esac
  echo "[PASS] $name exit=$rc effect=$eff"
}

# (1) publish-guard (FROZEN-PUBLISH, operator order 2026-11-02,
# docs/jira/v26.11.2/waves/publish-halt.md). The hook blocks publish
# commands UNCONDITIONALLY — the old authority-marker bypass
# (CGAP_PUBLISH_AUTHORITY_FILE) is superseded by the freeze, so the
# former marker-mutation court cannot flip and is retired here.
# Anti-vacuity mutation (house law: mutate /tmp sandbox copies, never
# the shared tree): a sed-neutered SANDBOX COPY of publish-guard.sh
# (case-guard replaced by `exit 0`) MUST flip block(2)->allow(0).
# If the mutant still blocks, the mutation did nothing => court VACUOUS.
court publish-guard 2 fire  "cargo publish -p cgap-core"
court publish-guard 0 quiet "cargo test -p cgap-core"
SB="$(mktemp -d)"
cp "$H/publish-guard.sh" "$SB/publish-guard.sh"
before_sha="$(shasum -a 256 "$H/publish-guard.sh" | cut -d' ' -f1)"
sed 's/^exit 2$/exit 0/' "$SB/publish-guard.sh" > "$SB/mutant.sh" || true
if ! cmp -s "$SB/publish-guard.sh" "$SB/mutant.sh"; then
  mout="$(printf "$J" "cargo publish" | bash "$SB/mutant.sh" 2>&1)"; mrc=$?
  if [ "$mrc" -eq 0 ]; then
    echo "[PASS] publish-guard sandbox-mutation: mutant flips block(2)->allow(0) (rc only; residual JSON inert under exit 0)"
  else
    echo "[FAIL] publish-guard sandbox-mutation did NOT flip (rc=$mrc out=$mout) — VACUOUS court"; fail=1
  fi
else
  echo "[FAIL] publish-guard sandbox-mutation sed did not diverge — VACUOUS court"; fail=1
fi
after_sha="$(shasum -a 256 "$H/publish-guard.sh" | cut -d' ' -f1)"
[ "$before_sha" = "$after_sha" ] || { echo "[FAIL] publish-guard shared-tree mutated!"; fail=1; }
rm -rf "$SB" 2>/dev/null || python3 -c "import shutil; shutil.rmtree('$SB', True)"

# (2) cwd-guard
court cwd-guard 0 fire  "cargo test --manifest-path /tmp/sandbox/Cargo.toml"
court cwd-guard 0 quiet "cd /tmp/sandbox && cargo test"
court cwd-guard 0 quiet "cargo test -p cgap-core"

# (3) mtime-restore-guard
court mtime-restore-guard 0 fire  "shasum -c manifest && cp /tmp/orig.rs restore/src/lib.rs # RESTORE"
court mtime-restore-guard 0 quiet "shasum -c manifest"

# (4) REP1 census gate — landed-law upgrade (this session).
# Law (docs/jira/v26.11.2/repairs/repair-01-branchless-census.md): the
# branchless census must run as REAL `#[test]`s under `cargo test -p
# cgap-branchless --test branchless_census`, NOT a hand-interpreted script —
# the census-cargo-test-blindness fix (REP1). The test's own anti-vacuity
# arms are the mutant classes this court pins by name:
#   - M-REP1-1: if-injection into a select.rs sandbox copy MUST flip
#     select_slice past its residual allowlist;
#   - M-INL-1: a data-dependent branch injected into an inline CALLER
#     probe sandbox copy MUST flip the caller's HOT cell off 0;
#   - branchy-reference self-check: `naive_if_cmov` MUST read BRANCHY.
# Outer tripwire semantics: exit 0 requires the full `test result: ok`
# line with >=3 passed (the 3 census fns: host hot cells, riscv32imc hot
# cells, mutation+selfcheck always-run arm). A compile-abort, renamed
# test binary, or silent skip reads as [FAIL], never a pass (census
# compile-abort-vs-failure law, same-checkout-fanout add. 3).
CENSUS_TARGET="$(mktemp -d "${TMPDIR:-/tmp}/g10-census-target.XXXXXXXX")"
# G10 root-cause fix (witnessed 2026-11-02): the census writes FIXED
# filenames into a FIXED shared dir (/tmp/rep1-census/obj_host.o,
# obj_riscv.o, mut_indep.o, ...). Two concurrent census invocations —
# which happens routinely under a multi-lane fan-out where each lane
# runs `cargo test -p cgap-branchless` — race on those files and rustc
# aborts with "could not copy ... to obj_host.o: No such file or
# directory", flipping the gate to a false [FAIL]. Serialize census
# invocations with a BLOCKING flock (kernel wait, no sleep-polling);
# fail-loud semantics are unchanged (a genuine census failure still
# exits nonzero inside the lock).
census_lock="${TMPDIR:-/tmp}/rep1-census.lock"
census_out="$(CARGO_TARGET_DIR="$CENSUS_TARGET" perl -e 'use Fcntl qw(:flock); open my $fh, ">", $ARGV[0]
  or die "lock open failed: $!"; flock($fh, LOCK_EX) or die "flock failed: $!";
  shift @ARGV; exec @ARGV' "$census_lock" cargo test -p cgap-branchless \
  --test branchless_census 2>&1)"; census_rc=$?
census_pass="$(printf '%s\n' "$census_out" \
  | sed -n 's/^test result: ok\. \([0-9]*\) passed.*/\1/p' | head -1)"
if [ "$census_rc" -eq 0 ] && [ -n "$census_pass" ] && [ "$census_pass" -ge 3 ]; then
  echo "[PASS] rep1-census-gate rc=0 passed=$census_pass (arms: M-REP1-1 / M-INL-1 / branchy-ref self-check)"
else
  echo "[FAIL] rep1-census-gate rc=$census_rc passed='${census_pass:-0}' :: $(printf '%s\n' "$census_out" | tail -3)"; fail=1
fi
rm -rf "$CENSUS_TARGET" 2>/dev/null || python3 -c "import shutil; shutil.rmtree('$CENSUS_TARGET', True)"

exit $fail
