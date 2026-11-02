# dxd-02-doctor-design

DXD-02 merged doctor surface design (2026-11-02). DESIGN only —
nothing landed. Grounded in fresh witness: doctor_agent_lanes.py
(4 checks, record() model, 0/1/2 max-exit), cli doctor.rs (10 checks,
same ABI), real dylib target/release/libcgap_ffi.dylib (615,552 B,
Nov 2 12:54, currently GREEN vs src), llvm-profdata NOT on PATH
(only xcrun -f), go1.23.4, pmset works.

## Decision: extend scripts/doctor_agent_lanes.py — one script

Not composed, not the cli bin: crates/cli is ggen-generated (regen
hazard); DX-1's Python idiom already has env-override courtability
and the witnessed-flip pattern; composing adds a third exit-ABI hop.
doctor.rs stays untouched (reference only).

## Exit ABI (unchanged)

0 all GREEN; 1 any WARN (informational); 2 RED (structural fault in
the check itself). Missing-but-expected-later = WARN, never RED.
Exit = max() over checks.

## Ordering: full report, no fast-fail

Total runtime <~5s cold; fast-fail hides co-failures (go AND profdata
both missing = both hints in one run). Display: DX-1's 4 checks first,
then new: riscv-target, profdata, go-version, ffi-dylib, shm-rule,
thermal.

## New check specs (each with anti-vacuity arm)

1. riscv-target: rustup target list --installed grep exact line;
   WARN if absent. Flip: DXD_TARGET_LIST env override with a stripped
   list -> YELLOW; with it -> GREEN.
2. profdata: shutil.which; WARN if absent (hint xcrun -f). Flip:
   stripped PATH -> WARN; augmented PATH -> GREEN. First real run is
   a witnessed WARN today.
3. go-version: parse go1.N.M, >=1.22 GREEN; missing = WARN (Rust-only
   sessions shouldn't hard-fail); unparseable = RED. Flip: DXD_GO_VERSION
   override — 1.21.5 YELLOW / 1.22.0 GREEN / garbage RED.
4. ffi-dylib: DXD_FFI_DYLIB default target/release path (IPC-0 W2);
   dual-path presence (lease dir fallback, report which); staleness vs
   newest of crates/cgap-ffi/src + Cargo.toml + include/cgap.h (header
   drift without rebuild = the false-pass class; NEVER vs the lease dir).
   Flip: /tmp dylib + utime(1,1) -> WARN stale; older marker -> GREEN;
   touch marker to now -> WARN. Triple witnessed pattern (DX-1 idiom).
5. shm-rule: Darwin -> GREEN (shm_open kernel-backed, no mount
   precondition); Linux -> real stat + test-open-for-write on /dev/shm
   (default DXD_SHM_DIR), missing/ro -> YELLOW "shm suite NO-GO per
   IPC-0 W5". Flip: DXD_FORCE_PLATFORM=Linux + fake dir -> YELLOW;
   read-only dir -> YELLOW; Darwin -> GREEN.
6. thermal: pmset -g therm parse (witnessed GREEN today); warn-level
   lines -> YELLOW "throttling likely; defer perf benches"; sysctl
   fallback; Linux arm UNVERIFIED on this host (disclosed). Flip:
   DXD_PMSET_OUT fixture file — clean -> GREEN, warning-level -> YELLOW.

## justfile wiring

Append-only `doctor-merged:` recipe calling the same script (the
script grows checks; the recipe shape does not change). Existing
doctor / doctor-agents untouched.

## Falsifier — stale dylib false-passing a cgo run

Why cgo false-passes alone: linking resolves the OLD dylib via
DYLD_LIBRARY_PATH; the 11-symbol table is a frozen superset so links
succeed; behavior-level changes don't change symbol names — tests
encoding old behavior pass against the wrong binary (exit 0, wrong
dylib); cgap_version may not even change. The doctor's mtime compare
vs in-tree src+header is the only assumption-free witness.
Concrete walk: /tmp dylib + older src marker -> GREEN; touch src newer
-> WARN "stale by N min" + remediation `cargo build --release -p
cgap-ffi`. Second falsifier: rebuild in the lease dir only — doctor
must still WARN (staleness always vs in-tree src, never the lease).

## Non-goals

No crates/cli changes; no new gates in just doctor's W2 chain;
Linux thermal arm UNVERIFIED; no staticlib expected (cdylib+rlib per
IPC-0 W2).

## See Also

audit/dxd-09-dual-runtime-env.md · lanes/lane-ipc-00-ground-truth.md ·
lanes/lane-dx-01-doctor-agent.md
