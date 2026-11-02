# lane-tps-05-battery

Witnessed: 2026-11-02 session. Lane TPS-5. Gate condition TPS-4 drained:
interop-chicago go surface green (stage 1a/1b exits 0, this session).

## W1 Files written

- `scripts/run_station300.sh` (NEW) -- TPS manual s3 battery, one
  fail-closed script; every command via `scripts/run_checked.py`
  (tees output, propagates the command's OWN rc). Stages 1a/1b go vet
  + go test; 2a build cgap-ffi lib; 2b cargo test workspace trio; 2c
  csct via --manifest-path (standalone crate, own [workspace] -- a
  `-p csct` selector in the root workspace fails with 'package ID
  specification did not match', witnessed /tmp/tps5-stage-2b.log);
  3 verify_disassembly_invariants.py; 4 verify_cscb_sram.py; 5 docker
  linux/arm64 row, typed [SKIP] if daemon dead.

## W2 Commands + exits (this session, exact)

| stage | command | exit | log |
|---|---|---|---|
| dry-run flip | `run_station300.sh --dry-run` (injects `sh -c 'exit 1'`) | 1 (halt, FLIP-CONFIRMED) | /tmp/tps5-stage-99.log |
| 1a | `go vet ./...` (courts/interop-chicago) | 0 | /tmp/tps5-stage-1a.log |
| 1b | `go test -count=1 ./...` | 0 | /tmp/tps5-stage-1b.log |
| 2a | `cargo build -p cgap-ffi --lib` | 0 | /tmp/tps5-stage-2a.log |
| 2b | `cargo test -p cgap-core -p cgap-git -p cgap-ffi` | 101 (FAIL, see W4) | /tmp/tps5-stage-2b.log |
| 2c | `cargo test --manifest-path crates/csct/Cargo.toml` | 0 (71 passed, 0 failed) | /tmp/tps5-stage-2c.log |
| 3 | `verify_disassembly_invariants.py` | 0 (AE4 court CLEAN) | /tmp/tps5-stage-3.log |
| 4 | `verify_cscb_sram.py` | 0 (36048 B <= 131072 B) | /tmp/tps5-stage-4.log |
| 5 | docker rust:1-bookworm arm64 `cargo test -p cgap-ffi` | 101 (45 passed, 5 failed, see W4) | /tmp/tps5-stage-5.log |

Two full battery invocations halted fail-closed at 2b (STATION-300-HALT
stage=2b rc=101, failing command named), as designed. Because the
battery halts at the first failing stage, stages 3-5 were additionally
run individually via the same run_checked.py harness to gather
evidence; each cites its own /tmp/tps5-stage-N.log.

## W3 Courts run

- dod manifest court (cgap-core manifest_court.rs) -- RAN, FAILED
  (see W4).
- AE4 disassembly-invariants court -- ALIVE (exit 0, 'CLEAN ... mutant
  + self-check + binary mutant all held').
- CSCB SRAM envelope gate -- ALIVE (exit 0, 36048/131072 B).
- interop-chicago go courts -- ALIVE (vet + test -count=1, exit 0).
- csct 13 test binaries, 71 tests -- ALIVE (0 failed).
- Anti-vacuity: --dry-run injected `sh -c 'exit 1'` halts rc=1 with
  STATION-300-DRYRUN-FLIP-CONFIRMED (witnessed this session).

## W4 Standing: PARTIAL

Two REAL, pre-existing failures on the shared tree, both blocking
battery green; neither is a TPS-5 defect and neither file is owned by
this lane:

1. `crates/cgap-core/tests/manifest_court.rs:46 dod10_no_spdx_license_identifiers_anywhere`
   FAILS against the manifest at HEAD, not just the working tree:
   `git show HEAD:crates/cgap-core/Cargo.toml` line 7 carries
   `license-file = "../../LICENSE.md"`, which the court's
   `line.contains("license-file")` assertion bans. The court contradicts
   the manifest's own frozen-license posture (the sibling court dod10_p28
   PASSES: URN present in both files). Needs an owning-lane decision
   (court assertion vs manifest law), not a lane-TPS-5 edit.
   Log: /tmp/tps5-stage-2b.log (panic at manifest_court.rs:56).
2. Docker linux/arm64 row: 45 passed, 5 FAILED -- all 5 are BSD-nm
   dependent (`nm -gU failed ... nm: 'a.out': No such file`,
   oabi_lookup.rs:328; wave9/wave_f2/wave_gap1 cdylib symbol courts).
   `nm -gU` is a macOS nm flag; linux nm rejects it. Environment-class,
   not logic: the same suite passes 2a+2b-class checks on the host.
   Log: /tmp/tps5-stage-5.log.

No STATION-300-PASS was emitted (correct: fail-closed, no digest over a
non-green transcript).

## W5 Divergences from the lane doc

- Dispatch named stage 2 as `cargo test -p cgap-core -p cgap-git -p csct
  -p cgap-ffi`; `csct` is a standalone crate (own `[workspace]`,
  crates/csct/Cargo.toml:15-16) so the root-workspace `-p` selector
  cannot see it. Script runs it as stage 2c via `--manifest-path`.
  Recorded here per doc-yields-to-reality law.
- Dispatch said 'CGO tests need the dylib -- use the documented build
  if needed'; cgap-ffi builds cdylib by default (crate-type cdylib+rlib,
  no feature gate), so stage 2a is the plain `cargo build -p cgap-ffi
  --lib`. There is no `--features cgo`.
- Docker stage 5 needed `export PATH=/usr/local/cargo/bin:$PATH` inside
  the container: `bash -lc` in rust:1-bookworm resets PATH via
  /etc/profile and loses cargo (witnessed exit=127 first attempt,
  /tmp/tps5-stage-5.log superseded by the fixed rerun, same path).

## W6 Fence law

Must-not-regress surfaces this diff could break: none outside owned
files -- the script only READS the tree (cargo/go/docker run against
the existing build) and writes /tmp logs. Shared tree untouched by any
write. The docker stage mounts the repo read-... (not read-only: cargo
writes /work/target inside the container; noted, container removed
--rm).

## W7 Evidence (kept, not cleaned)

/tmp/tps5-stage-{99,1a,1b,2a,2b,2c,3,4,5}.log,
/tmp/tps5-battery-stdout.log (full-battery halt transcripts),
/tmp/tps5-{2c,3,4,5}-wrap.log.

Script final sha256: see `shasum -a 256 scripts/run_station300.sh` at
receipt-landing time.
