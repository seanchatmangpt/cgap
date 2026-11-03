# V2 Release Gates G1-G8 (Gate-Runner Dispatch Blocks)

Charter: CHARTER-CGAP-STARDF-v26.11.2, section GOVERNING (cited, not redefined here).
Lease: docs/jira/v26.11.2/01-release-gates.md (this file) + waves/v2-02-gates.md (receipt).

Gate-runner dispatch contract (every block carries all four fields):

- COMMAND: exact shell, run from `/Users/sac/cgap` unless the block says otherwise.
- RECEIPT: the required quantitative artifact; a pass without the number is not a pass.
- FLIP: anti-vacuity court. A court that cannot flip is FAIL. Skips are loud
  (`exit 3` + `[SKIP] <reason>` line in the receipt), never silent.
- BLOCKED: typed protocol when the subject is missing or the environment cannot run it.

Exit-code contract (repo CLAUDE.md): 0 pass / 1 gate fail / 2 subject missing /
3 loud skip. Gates mutate only /tmp sandbox copies with sha256-exact restore;
never the shared tree. No Process.sleep anywhere in these gates.

---

## G1-PUBLISH

COMMAND:

    python3 scripts/release_check.py
    # REQUIRED EDIT before first run: scripts/release_check.py:21 pins
    # REQUIRED_VERSION = "26.10.29"; must read "26.11.2" for this release.
    # Gate set = 19/22 workspace members; court/cli/bench are lawfully
    # delisted (publish = false), beam optional (see BLOCKED).

RECEIPT:

- 19/22 crates publishable set matches delisting list; per-crate
  `cargo package --no-verify --allow-dirty -p <crate>` exit 0.
- .crate sizes + file lists from `cargo package --list`, recorded per crate.
- Workspace version reads exactly `26.11.2` in root [workspace.package].

FLIP:

- In a /tmp sandbox copy: strip `description` from one publishable crate's
  Cargo.toml, run release_check on the copy; the gate must FAIL (exit 1).
- sha256 the sandbox Cargo.toml before/after; fail-closed restore.

BLOCKED:

- `cargo package` network failure or crates.io auth missing -> exit 2
  (subject missing: publish transport), record `[SKIP] publish-transport`.
- Workspace version still 26.11.1 on disk today (observed 2026-11-01) ->
  gate self-fails (exit 1) until the version bump lands; do not override.

---

## G2-PARETO

COMMAND (per level, from /Users/sac/cgap; run with `--profile bench` defaults):

    cargo bench -p bench --bench page_bench        # L1 frames: report ns/frame
    cargo bench -p bench --bench dict_bench        # L1/L2 dict paths
    cargo bench -p bench --bench parquet_bench     # L2 1M-row scans
    cargo bench -p bench --bench bitslice_bench    # L2/L3 bit filter paths
    cargo bench -p bench --bench packing_bench     # L3 pack throughput
    cargo bench -p bench --bench parquet_throughput  # L5 rows/s
    # ns/frame from target/criterion/*/new/estimates.json (median), never from
    # stdout alone; scripts/divan2receipt.py is the parser of record.

RECEIPT (all five must be numbers in the receipt, with bench identity):

- L1 < 15 ns/frame
- L2 < 10 ms @ 1M rows
- L3 > 20x @ 1M (vs L2 scalar baseline, same machine, same run)
- L4 < 4 us/tx
- L5 < 1.45 us/row

FLIP (mandatory per level, /tmp sandbox copy of the measured kernel):

- Mutate the kernel (e.g. remove `#[inline]` / inject a bounds check /
  add a temporary `core::hint::black_box` around the hot loop) in a /tmp
  copy, re-run the single bench, and require the level's threshold to
  flip to FAIL. sha256-exact restore of the mutated file.
- A level with no flip witness is not measured; it is [SKIP] loud.
- Restore pattern witnessed: waves/w2-13-l4l5-courts.md:80.

BLOCKED:

- Bench target missing for a level -> exit 2, `[SKIP] <level> bench-absent`.
- Threshold met without a flip witness -> receipt line
  `VACUOUS: L<n> threshold met with no flip court` and exit 3.

---

## G3-MAT-BENCH

COMMAND:

    cargo bench -p bench --bench <mat_closure_bench>
    # Like-for-like constraint: same metric class as RDFox 2-6.1M facts/s
    # (derived facts per second per core, materialization, single thread).
    # Pin: single core (taskset-equivalent: Rust `std::thread` x1, pinned
    # via criterion `sample_size` config in the bench source), warm cache,
    # same fact-count buckets as the RDFox citation baseline.

RECEIPT:

- > 50M closure facts/s/core, single-core, like-for-like metric class.
- Receipt names the RDFox baseline figure used (2-6.1M facts/s), its
  source, and the machine/method line for both sides.

FLIP:

- Mutate the closure kernel in a /tmp sandbox copy to skip one derivation
  rule (or de-duplicate one join arm) so derived-fact count changes; the
  bench must detect the changed fact count or the throughput delta must
  flip the threshold. sha256-exact restore.

BLOCKED:

- No mat closure bench exists yet in crates/bench (observed 2026-11-01:
  no `mat` target in crates/bench/Cargo.toml [[bench]] list) -> exit 2,
  `[SKIP] mat-closure-bench-absent`.

---

## G4-DRED

COMMAND:

    cargo bench -p bench --bench <dred_bench>
    # Subject: DRED (docs/jira/v26.11.1/waves/paper-dred.md).
    # Churn protocol: 90% churn workload (90% of facts retracted+reinserted
    # per batch), single batch = one full increment+mat cycle.

RECEIPT:

- < 3.0 ns/batch (batch = full increment+mat cycle at 90% churn)
- < 0.15 ns entropy variance (spread across >= 9 batches)
- > 15M facts/s at 90% churn
- Batch count N >= 9 with per-batch numbers in the receipt, not just a mean.

FLIP:

- Mutate the DRED increment path in a /tmp sandbox copy to skip the
  entropy recompute; the entropy-variance number must change and the
  <0.15 ns variance gate must flip. sha256-exact restore.

BLOCKED:

- No dred bench target exists (observed 2026-11-01: absent) -> exit 2,
  `[SKIP] dred-bench-absent`. paper-dred.md is prose, not a bench.

---

## G5-SILICON

COMMAND:

    rustup target add riscv32imc-unknown-none-elf
    bash scripts/footprint.sh
    python3 scripts/symbol_gate.py
    # QEMU stack-paint: build the no_std binary with stack-paint fill,
    # run under qemu-system-riscv32, read the high-water mark.
    qemu-system-riscv32 -M <board> -nographic -kernel <elf>
    # <board> and ELF path come from the wasm-footprint/footprint harness
    # output; record both in the receipt.

RECEIPT:

- flash < 32,768 B (size -A on the riscv32imc ELF)
- SRAM + stack < 4,096 B (size -A .bss+.data + QEMU stack high-water)
- 0 alloc/panic symbols (nm | grep -c alloc/format/panic == 0)
- QEMU stack-paint high-water number recorded.

FLIP:

- In a /tmp sandbox copy, add a `Vec::new()` push (alloc symbol) or a
  `panic!` site to the no_std subject; symbol_gate.py must FAIL on the
  copy. sha256-exact restore.

BLOCKED:

- riscv32imc target or qemu-system-riscv32 absent -> exit 2,
  `[SKIP] riscv32/qemu-absent`.
- footprint.sh / symbol_gate.py subjects are wasm-class today; if the
  riscv32 path is not wired in the harness, exit 2 `[SKIP] riscv-unwired`.

---

## G6-NO-DF

COMMAND:

    grep -ri "datafusion" \
      /Users/sac/cgap/Cargo.toml /Users/sac/cgap/Cargo.lock \
      /Users/sac/cgap/crates/*/Cargo.toml /Users/sac/cgap/crates/*/src \
      /Users/sac/purrdf/Cargo.toml /Users/sac/purrdf/Cargo.lock \
      /Users/sac/purrdf/crates/*/Cargo.toml /Users/sac/purrdf/crates/*/src
    test "$(grep -ri datafusion <same-args> | grep -cv '^\s*#')" -eq 0

RECEIPT:

- Zero datafusion matches (comments included; a comment hit is a fail)
  across both repos' manifests, locks, and crate sources.
- Current state observed 2026-11-01: /Users/sac/cgap/Cargo.toml:33 carries
  a stale comment mentioning "DataFusion" (CD-1 note, no dependency;
  Cargo.lock count = 0). G6 exits 0 only once that comment is gone.

FLIP:

- /tmp mutation flip: append `datafusion = "42"` to a /tmp copy of a
  workspace Cargo.toml and rerun the gate on the copy; it must FAIL.
- Self-check corrupts a restore on a real temp copy (gate refuses a
  restore mismatch).
- Self-check precedent on record: waves/w2-08-gate-reruns.md.

BLOCKED:

- Not applicable; G6 is fully runnable today (only the stale comment and
  a placeholder-free mutation harness are needed).

---

## G7-PATENT

COMMAND:

    python3 verify_ost_design_around.py \
      --claims docs/jira/v26.11.2/patent/claims.txt \
      --corpus /Users/sac/purrdf/corpora \
      --report docs/jira/v26.11.2/patent/ost_report.json
    # Auditor sign-off: RDF-11 auditor records a sign-off line in the receipt
    # (auditor identity + date + 4/4 DISTINCT-MECHANISM verdict).

RECEIPT:

- 4/4 claims DISTINCT-MECHANISM (script exit 0, per-claim verdict rows).
- RDF-11 auditor sign-off line, named auditor, dated.

FLIP:

- Feed the script a mutated claims file where one claim is merged with a
  prior-art mechanism (near-duplicate); the verdict must drop to 3/4 and
  the gate must FAIL. sha256-exact restore of the claims file.

BLOCKED:

- `verify_ost_design_around.py` does not exist in either repo (observed
  2026-11-01: not found under /Users/sac/cgap or /Users/sac/purrdf) ->
  exit 3, `[SKIP] verify_ost_design_around.py-absent`. This is the
  loud-skip path, not a pass. Gate stands BLOCKED until the script and
  the claims.txt subject land.

---

## G8-STARDF-F1

COMMAND (from /Users/sac/purrdf):

    python3 scripts/stardf_f1_census.py        # F1 census, exit 0
    # 11 mappings added: census output counts mapping rows == 11.
    # Golden digests: 10/10 vs purrdf golden corpus.
    python3 scripts/stardf_golden.py --expect 10
    # KD-12/KD-14 differential courts:
    python3 scripts/kd_flip.py --court KD-12
    python3 scripts/kd_flip.py --court KD-14

RECEIPT:

- Census exit 0 with exactly 11 new mapping rows.
- 10/10 golden digests byte-match the purrdf golden corpus.
- KD-12 and KD-14 differential courts each show a witnessed flip.

FLIP:

- KD-12/KD-14 courts are themselves the flip courts: remove one of the 11
  mappings in a /tmp sandbox copy of the mapping table and require the
  census to exit nonzero and the golden digest to break. Restore
  sha256-exact.
- Restore pattern witnessed: waves/w2-13-l4l5-courts.md:80.

BLOCKED:

- scripts/stardf_f1_census.py / stardf_golden.py / kd_flip.py do not
  exist under /Users/sac/purrdf/scripts today (observed 2026-11-01) ->
  exit 2, `[SKIP] stardf-census-scripts-absent`. Gate stands BLOCKED
  until those subjects land in purrdf.

---

## Runner wiring

- These blocks are the dispatch surface for `gate-runner` (lane-runner
  agent, one gate per dispatch). Each block is self-contained: the runner
  reads COMMAND/RECEIPT/FLIP/BLOCKED verbatim.
- File ownership: this lane owns
  docs/jira/v26.11.2/01-release-gates.md and
  docs/jira/v26.11.2/waves/v2-02-gates.md only.

## Session evidence (2026-11-01)

- `grep -ri datafusion` over both repos: single hit =
  /Users/sac/cgap/Cargo.toml:33 (stale CD-1 comment; Cargo.lock count 0).
- Workspace version observed: 26.11.1 (root Cargo.toml [workspace.package]),
  not the 26.11.2 that G1 requires.
- crates/bench [[bench]] list: no mat-closure or DRED target.
- verify_ost_design_around.py: absent from both repos.
- /Users/sac/purrdf exists; purrdf/scripts lacks
  stardf_f1_census.py / stardf_golden.py / kd_flip.py.

---

## G1-PUBLISH flag table — refreshed re-derivation (2026-11-02)

Evidence receipt: `docs/jira/v26.11.2/audit/pub-02-flag-table-rederivation.md`
(PUB-02; zero-memory re-derivation: cargo metadata, on-disk grep of
`publish` flags, 23 sequential crates.io API GETs with mandatory UA).
ERRATA (superseded, history retained): the G1 block above carries
"Gate set = 19/22 workspace members" (:27) and "19/22 crates
publishable set" (:32). Both figures are stale by drift — fresh
derivation is 23 members, 20 publish-true / 3 publish-false. Drift
sources: cgap-fibo/cgap-cloud are members living under `ontologies/`
(not `crates/`), and cgap-branchless flipped to `publish = true` by
DOD-02 (`lanes/lane-dod-02-branchless-pub.md`). The CLAUDE.md figure
"12/23 explicit, 9 true / 3 false" is likewise superseded by
23/23 explicit. The stale text above is retained verbatim per
ERRATA-never-rewrite law; this table governs.

### Member table (23/23 explicit: 20 true / 3 false)

| # | crate | flag | registry | max_version |
|---|-------|------|----------|-------------|
| 1 | cgap | true | exists | 26.11.2 |
| 2 | cgap-arrow | true | exists | 26.11.2 |
| 3 | cgap-core | true | exists | 26.11.2 |
| 4 | cgap-blake3 | true | exists | 26.11.2 |
| 5 | cgap-gate | true | exists | 26.11.2 |
| 6 | cgap-branchless | true | 404-pending | registry-absent (DOD-02 first-publish) |
| 7 | cgap-storage | true | exists | 26.11.2 |
| 8 | cgap-git | true | exists | 26.11.2 |
| 9 | cgap-bn3 | true | exists | 26.11.2 |
| 10 | cgap-parquet | true | exists | 26.11.2 |
| 11 | cgap-p28 | true | exists | 26.11.2 |
| 12 | cgap-crypto | true | exists | 26.11.2 |
| 13 | cgap-ffi | true | exists | 26.11.1 (lag) |
| 14 | cgap-audit | true | exists | 26.11.1 (lag) |
| 15 | cgap-beam | false | 404 | (moot: publish=false) |
| 16 | cgap-gpu | true | exists | 26.11.1 (lag) |
| 17 | bench | false | collision | 1.1.0, ours-not (bench-rs, 2021) |
| 18 | cgap-integration | true | 404-pending | registry-absent |
| 19 | cgap-bont | true | exists | 26.11.2 |
| 20 | cgap-bosym | true | exists | 26.11.2 |
| 21 | cgap-court | false | 404 | (moot: publish=false) |
| 22 | cgap-fibo | true | exists | 26.11.1 (lag) |
| 23 | cgap-cloud | true | exists | 26.11.1 (lag) |

Registry reads are HTTP codes from the PUB-02 session (2026-11-02);
five members at 26.11.1 are pre-version-bump publications (expected
pre-publish-wave lag, not defects). Note on cgap-integration: flag
read from `crates/integration/Cargo.toml:8` (dir name ≠ pkg name).

### Non-member (standalone) crates — 2026-11-02

| crate | flag | registry |
|-------|------|----------|
| cgap-verification | false (by design) | not read (publish=false, moot) |
| csct | false (by design) | not read (publish=false, moot) |
| cli | false (by design) | not read (publish=false, moot) |
| cgap-fleet-ops | false (by design) | not read (publish=false, moot) |
| cgap-ocel2 | false (by design) | not read (publish=false, moot) |
| wasm-footprint | true | UNVERIFIED-registry (no read performed) |

wasm-footprint is a publish=true non-member outside the 23-member
gate set — a live first-publish candidate the prior table omitted
(PUB-02 W5). Its registry state carries no HTTP read; it is
UNVERIFIED, never rounded up. The five publish=false standalone
crates are delisted by design, per the PUB-02 disk reads.

Standing: ALIVE (2026-11-02) — every member-table cell re-verified
against `audit/pub-02-flag-table-rederivation.md` this session;
cgap-integration flag additionally re-read from disk
(`crates/integration/Cargo.toml:8`, `grep -n "^publish"`, exit 0)
after the PUB-02 W4 annotation typo was found. This lane owns only
this file; no other tree file was edited; no git commands run.
