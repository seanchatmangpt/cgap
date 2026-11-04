# cbmqs-runner

The CBMQS bake-off RUNNER (AF-14): executes the 7-gate CBMQS standard
against an arbitrary SUBJECT and emits a machine-readable, BLAKE3-sealed
verdict report. Exit codes: 0 = PASS, 2 = FAIL, 3 = REFUSED (mock
framework detected), 1 = usage/infrastructure error.

Binary: `crates/cgap-aoq/tests/cbmqs_runner.rs` — a `harness = false`
test target (see `crates/cgap-aoq/Cargo.toml`), so the process exit
status IS the verdict (repo exit-record law). Build+run:

```bash
cargo test -p cgap-aoq --test cbmqs_runner            # self-test mode
cargo test -p cgap-aoq --test cbmqs_runner -- --court # non-vacuity court
cargo test -p cgap-aoq --test cbmqs_runner -- \
  --subject-cmd "python3 /tmp/subject.py" \
  --source-tree /path/to/subject/tree \
  --gates G1,G2,G5,G6 \
  --alloc-bound 0 --time-budget-ns 10000000 \
  --json /tmp/report.json
```

## Gates

| Gate | Law | Self-test form | Foreign form |
|---|---|---|
| G1 | alloc-per-op census ≤ bound (default 0) | counting global allocator around 8192 real encode+decode roundtrips; positive control (a definitely-allocating window must show allocs ≥ 1, else `VACUOUS_ALLOC_CENSUS`) | CENSUS frames around 512 OP frames |
| G2 | timing census vs budget | per-op Instant census, p50/p99 over 8192 ops | 16 batches x 64 OP frames |
| G3 | anti-rigging mock scan | scans `crates/cgap-aoq/src` | scans `--source-tree` (typed `SOURCE_TREE_REQUIRED` if absent) |
| G4 | census replay determinism | two census windows byte-identical | two census exchanges identical |
| G5 | storm feed: partial-accept + oversized | exact typed classes against the real codec (`HeaderTooShort`, `DigestMismatch`, `BadMagic`, `UnknownEpochWire`) | contract-typed: mandated-refusal frames (wrong length/magic) must elicit typed `ERR <CLASS>`, never crash; digest-class frames are subject-law-dependent (`Expect::OkOrTypedErr`) |
| G6 | kill-cycles | typed `SKIP(SUBJECT_IS_LIBRARY)` (machinery witnessed live in court) | 3 cycles of PING → SIGKILL → respawn → PING |
| G7 | report integrity | BLAKE3 seal over the canonical body + runner version stamp | same |

Self-test verdict PASS (G6 typed SKIP disclosed) is the anti-vacuity
proof that the standard is satisfiable. `--court` proves every gate can
flip: overalloc fixture fails `OVERALLOC`, missing-census fixture fails
`CONTRACT_UNSUPPORTED`, bound-flip pair (same subject FAILs at bound 0,
PASSes at bound 2), storm/kill-cycle fixtures, mock-tree refusal, and a
tampered report must refuse the seal.

## Subject contract (subprocess mode)

Line protocol on stdin/stdout (UTF-8):

- `PING` → `PONG`
- `CENSUS` → `CENSUS allocs=<u64> bytes=<u64>` (self-attested allocation
  census; if the subject cannot census itself, reply `ERR NO_CENSUS` →
  G1/G4 fail `CONTRACT_UNSUPPORTED`, loudly missing, never a silent skip).
- `OP <n>` → perform n advertised ops; reply `OPDONE <n>`.
- `FRAME <hex>` → apply the subject's own codec law; reply `OK` or
  `ERR <CLASS>`, never crash.

Contract disclosure: foreign G1 trusts subject-attested census. An
adversarial subject could lie to CENSUS; adversarial verification of a
foreign binary requires LD_PRELOAD allocator interception or source
instrumentation — registered as a known limitation, disclosed rather
than papered over.

## Relation to the CBMQS standard (AF-13)

AF-13's landed self-standard is `crates/cgap-aoq/tests/cbmqs_gates.rs`
(composes the LANDED courts: AB-9 alloc, AB-19 slab, AB-6 bench + drift,
AB-2/AB-3 KATs, AB-10/AB-11 storm, AB-17 churn, AB-14/AF-7 receipt
chain). This runner is the generic-subject EXECUTOR: the same gate IDs,
generic forms for arbitrary subjects. Registered divergences:

- Runner G2 budget is measured in the debug profile; AF-13's G2 bound is
  RELEASE-only. Self-test p99 was 917-1375 ns (budget 10000 ns).
- Runner G6 = kill-cycles (generic); AF-13's G6 = AB-17 200-cycle churn
  (witnessed PASS-CITED cross-repo in `cbmqs_gates.rs`).
- AF-13 has an extra G1b (slab throughput); the runner's standard is
  CBMQS-7G/1 (7 gates, no G1b).

## See Also

- `crates/cgap-aoq/tests/cbmqs_runner.rs` — the runner source
- `crates/cgap-aoq/tests/cbmqs_gates.rs` — AF-13 self-standard evaluator
- `docs/jira/v26.11.3/receipts/af-14-cbmqs-runner.md` — receipt
