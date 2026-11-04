# cgap — one-command DX recipes (wave-10 lane 3)
#
#   just doctor   - toolchain/env health probe via cgap-cli doctor bin (lands with wave-10 lane 2)
#   just verify   - full v26.10.31 verification ladder (scripts/verify_v26.10.31.sh)
#   just bench    - release-build the bench crate, then run the CTQ report binary
#   just courts   - 3 court suites sequentially with headers, fail-fast
#   just receipts - newest receipts + Standing tally from docs/jira/v26.10.31/receipts/
#   just dod      - Definition-of-Done gate (scripts/dod.sh; lands with wave-10 lane 1)
#   just status   - portfolio status bin via cgap-cli status (lands with wave-10 lane 11)
#
# Every recipe wires an existing script/bin — no logic duplicated here. `doctor`,
# `dod`, and `status` target artifacts owned by other lanes and 404 until those
# lanes land, by design.

default:
    @just --list

# Toolchain/env health probe + W2 gate chain (W2-14).
# Exit classes: gate 0 -> GREEN, 1 -> RED (chain aborts), 2/3 -> YELLOW loud [SKIP].
# Chain exit: 0 all-GREEN / 1 any-YELLOW / 2 any-RED (doctor.rs model).
doctor:
    @rc_total=0; \
    cargo run -q --manifest-path crates/cli/Cargo.toml --bin doctor; rc=$?; \
    if [ $rc -ge 2 ]; then exit 2; fi; \
    if [ $rc -eq 1 ]; then rc_total=1; fi; \
    for g in zero_datafusion_gate receipt_lint registry_sweep; do \
      python3 scripts/$g.py --root . >/tmp/w2-14-$g.out 2>&1; rc=$?; \
      case $rc in \
        0) echo "$g: GREEN" ;; \
        1) echo "$g: RED (gate refuse)"; cat /tmp/w2-14-$g.out >&2; exit 2 ;; \
        2|3) echo "$g: YELLOW [SKIP] rc=$rc"; cat /tmp/w2-14-$g.out >&2; rc_total=1 ;; \
        *) echo "$g: RED (unexpected rc=$rc)"; exit 2 ;; \
      esac; \
    done; exit $rc_total

# Full v26.10.31 verification ladder
verify:
    bash scripts/verify_v26.10.31.sh

# Release-build the bench crate, then run the CTQ report
bench:
    cargo build --release -p bench
    target/release/ctq_report

# All 3 court suites, sequential, fail-fast, one header each
courts:
    @echo "== court 1/3: validate_ontology =="
    bash scripts/validate_ontology.sh
    @echo "== court 2/3: audit_branchless --selftest =="
    bash scripts/audit_branchless.sh --selftest
    @echo "== court 3/3: cfg_homology --mutant =="
    bash scripts/cfg_homology.sh --mutant

# LSPA-6: LSP tier-2 aarch64 wire court + AE4 disasm court (staleness-guarded)
branchless:
    python3 scripts/verify_lsp_branchless.py

# Newest 20 receipts + Standing tally (STANDING template row excluded by case)
receipts:
    @ls -lt docs/jira/v26.10.31/receipts/ | head -20
    @echo ""
    @echo "Standing tally:"
    @grep -h '| Standing |' docs/jira/v26.10.31/receipts/*.md \
        | grep -v STANDING \
        | awk -F'|' '{gsub(/^[ \t]+|[ \t]+$/, "", $3); c[$3]++} END {for (s in c) printf "  %-24s %d\n", s, c[s]}' \
        | sort

# Definition-of-Done gate (script owned by wave-10 lane 1; fails until landed)
dod:
    @if [ -f scripts/dod.sh ]; then bash scripts/dod.sh; else echo "dod: scripts/dod.sh not present yet (wave-10 lane 1 owns it)" >&2; exit 1; fi

# Portfolio status bin (bin owned by wave-10 lane 11; fails until landed)
status:
    cargo run -q --manifest-path crates/cli/Cargo.toml --bin status

# Lease janitor: scan /tmp for lane build-root leases (DX-3); dry-run default,
# `just leases --clean` deletes orphaned leases only (grammar fail-closed).
leases:
    python3 scripts/lease_janitor.py

# DX-1: per-lane doctor checks (stale-server, orphan leases, workspace count, dirty census).

# DX-1: per-lane doctor checks (stale-server, orphan leases, workspace count, dirty census).
# Exit model as `doctor`: 0 GREEN / 1 any-WARN informational / 2 RED.
doctor-agents:
    @python3 scripts/doctor_agent_lanes.py; rc=$?; if [ $rc -eq 2 ]; then exit 2; fi; exit $rc

# DX-8: per-target test census (compile-abort vs real-FAIL classification).
# Usage: just census [ARGS="--crate cgap-git,cgap-parquet"]. Exit 1 only on real FAILs;
# COMPILE-ABORT prints LOUD-PENDING and exits 0 unless ARGS includes --strict.
export CENSUS_ARGS := ''
census:
    python3 scripts/test_census.py {{CENSUS_ARGS}}

# CG-1 (v26.11.3, gate G7): wire-shm lane — real test suite + zero-dep proof.
# `cargo tree --depth 0` printing ONLY the `wire-shm` root line is the
# mechanical zero-external-dependency proof (any dep edge shows at depth 1+;
# a dep added without this recipe noticing is a RED refuse, not a warning).
# `just doctor` rides along as INFORMATIONAL (its exit is not this recipe's
# verdict; wire-shm health is decided by the test suite + tree proof).
wire-shm:
    cargo test --manifest-path crates/wire-shm/Cargo.toml
    tree_out=`cargo tree --manifest-path crates/wire-shm/Cargo.toml --depth 0`; \
    echo "$tree_out"; \
    lines=`printf '%s\n' "$tree_out" | wc -l | tr -d ' '`; \
    if [ "$lines" != "1" ]; then echo "wire-shm: RED — depth-0 tree shows $lines lines; external deps present" >&2; exit 1; fi; \
    printf '%s\n' "$tree_out" | grep -q "^wire-shm " || { echo "wire-shm: RED — root line missing" >&2; exit 1; }; \
    echo "wire-shm: GREEN (zero external deps proven)"; \
    just doctor-info

# Informational doctor probe (never decides this lane's exit; G7 contract).
doctor-info:
    @just doctor >/tmp/cg1-doctor.out 2>&1; rc=$?; \
    echo "doctor (informational): rc=$rc"; exit 0
