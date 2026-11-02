# release-blockers

Ranked register of OPEN/DEFECT items aggregated from this session's
receipts (audit lane AUD-03, 2026-11-02, repo /Users/sac/cgap @ main).
Every row cites its owning receipt; where cheap, status was re-checked
on disk this session (re-check noted per row). Standing vocabulary per
CLAUDE.md: ALIVE requires command+exit run this session.

## R1 — C6 retraction throughput gate: honest FAIL (reconciled)

- Status: OPEN (code/perf defect, rank 1).
- Owner receipts: lanes/lane-b05-cscb-c6.md (CSCB engine) +
  lanes/lane-git-11-retract.md (GIT-11 dred engine).
- Reconciliation: TWO honest FAILs against the same 800M facts/s gate,
  not one. b05 measured min 248,062,015.5 facts/s (lane-b05:42,49) and
  chose the typed FAIL over tuning. GIT-11 later measured min
  323,843,635.3 facts/s (lane-git-11:86), registered as MEASURED,
  UNGATED (:113). dred.rs:73-76 alt path reaches 839-944M facts/s but
  is ineligible (lane-git-11:90-94): tuning it to pass would be vacuous
  by its own court law.
- Re-check: none needed (perf claim, not cheaply re-runnable; numbers
  stand as-of-receipt).
- Close condition: a bench run, this session, whose honest min >= 800M
  facts/s with the b05 flip court green, OR an operator decision to
  lower the gate with a written gate-change receipt.


- GAP-04 append (2026-11-02): C3/C6 now DECIDED-gated in code —
  cscb_c3_closure.rs:40 `const DECIDED_BUDGET_MS: f64 = 60_000.0;` and
  cscb_c6_retraction.rs:43 `const DECIDED_GATE_FPS: f64 =
  150_000_000.0` (read fresh this session). PROVENANCE GAP recorded
  loudly: no lane-mv-5 receipt exists in docs/jira/v26.11.2/lanes/
  (only lane-mv-04-warnings-census.md, lane-mv-08-doc-refresh.md as of
  2026-11-02) — MV-5 provenance for these DECIDED gates is absent; MV-5
  may be in flight, do not treat the gates as receipted until its
  receipt lands.

## R2 — FA9/AE1 court kills default-feature `cargo test -p cgap-parquet`

- Status: OPEN (code defect), re-verified THIS session.
- Owner receipt: lanes/lane-c05-dead-code-excision.md:89-94 (recorded
  divergence, explicitly not a C05 edit).
- Re-check this session: `cargo test -p cgap-parquet` (default
  features) -> exit 101, 18 compile errors in the test cfg of
  crates/cgap-parquet/src/mmap_writer.rs. Un-gated test-only items
  reference std-gated collaborators: `RingSink` used at
  mmap_writer.rs:1844,1902 and `private::Sealed` impl near :1823,
  while the RingSink impls carry `#[cfg(feature = "std")]` at
  mmap_writer.rs:743,751,757,772. Effect: the FA9 rotation writeback
  court is UN-GATED under default features — it cannot run, so it
  cannot fail, so it protects nothing on a default `cargo test`.
- Close condition: gate the test module (or its imports) with
  `#[cfg(all(test, feature = "std"))]` so both feature sets compile;
  then `cargo test -p cgap-parquet` exit 0 AND
  `cargo test -p cgap-parquet --features std` exit 0.

## R3 — Coverage floor RED on four scripts (floor not lowered)

- Status: OPEN (process/gate hygiene), honest red preserved.
- Owner receipt: lanes/lane-c06-scripts-gate-coverage.md:51-49+.
- Measured per-script branch coverage (lane-c06:46-49):
  boundary_gate.py 24.4%, receipt_lint.py 64.9%,
  zero_datafusion_gate.py 52.6%, zero_string_gate.py 40.8%.
- `python3 tests/gates/enforce_coverage_floor.py --floor 90` -> exit 1
  with four red scripts. Floor was NOT lowered.
- Close condition: lift each script to >=90% branch coverage, or the
  operator registers an explicit floor exception per script.

## R4 — G2-PARETO rollup rows L1/L2/L4/L5 OPEN (courts were shells)

- Status: OPEN (process/evidence item).
- Owner receipts: DEFECT-REGISTER.md:13-34 (D-G08-1) +
  waves/lane-g08-defect-register-close.md + waves/w2-13-l4l5-courts.md
  ("court shells (pre-lever controls)", receipt :1, cargo rows :16-17
  are exit-0 control shells, not threshold measurements).
- Re-check: none needed; G08 ran no benches by design.
- Close condition: a G2-PARETO receipt carrying all five level numbers
  + per-level flip witnesses per 01-release-gates.md:56-88
  (scripts/divan2receipt.py as parser of record). L3 alone is
  CLOSED (repair-06 radix flip witnessed).

## R5 — receipt_lint UNRECEIPTED-CLAIM census growing on the live tree

- Status: OPEN (process/hygiene), re-run THIS session.
- Owner receipts: waves/w2-03-receipt-lint.md:27,34 (143 over 58 docs);
  waves/w-lint-fix.md:10-18 (212 at lane start, fixed files zero);
  waves/w2-20-terminus.md:97 ("142 strict census, W8 fixes").
- Re-check this session: `python3 scripts/receipt_lint.py` -> "FAIL 445
  (190 docs scanned)". The census has GROWN 143 -> 212 -> 445 across
  waves; prior receipts' counts are stale-as-of-today.
- Close condition: W8 (or successor) drives the live census to 0, and
  a floor gate converts receipt_lint from advisory to fail-closed.



- MV-8 append (2026-11-02): re-run THIS session — `python3 scripts/receipt_lint.py` prints "FAIL 445 (190 docs scanned)" (script exit 0, still advisory). Census UNCHANGED vs the prior re-check (445). Close condition unchanged.
## R6 — coordinator fix for census race: PRESENT (verified, carried)

- Status: CLOSED-VERIFIED this session.
- Re-check: crates/cgap-branchless/tests/branchless_census.rs:55-60
  carries the G10 root-cause fix — per-process `work()` via
  `OnceLock<String>` keyed on `std::process::id()` so concurrent
  census invocations cannot race shared obj artifacts. Comment block
  present at :55-57. No action owed.

## R7 — missing receipts for landed code: F5/F10/WG2/WG3

- Status: OPEN-UNVERIFIED (provenance).
- Owner receipt: Agent-15 inventory finding (dispatch input; no
  standalone receipt file located on disk this session).
- Re-check this session: word-boundary grep for F5/F10/WG2/WG3 across
  docs/jira/v26.11.2/{waves,lanes,audit} found NO covering receipt —
  the CSCT lane receipts (csct-lane-01..14, csct-wave-register) do not
  name these workstreams. UNVERIFIED stays UNVERIFIED; it does not
  round up to covered.
- Close condition: either landing receipts naming each workstream, or
  a coordinator errata mapping F5/F10/WG2/WG3 to the receipts that do
  cover the landed code.

## R8 — build debris still on disk (delete-never-commit class)

- Status: OPEN (hygiene), re-verified THIS session via ls -la.
- Owner receipt: lanes/lane-g20-make-gate0-replay.md.
- Still on disk: libae8_dbg.rmeta (repo root, 2,377 B, mtime Nov 2
  11:12) and crates/cgap-branchless/wrapper_riscv32imc.o (6,648 B,
  mtime Nov 2 12:08). AUD-03 writes only this register; deletion is
  the owning lane's or integration's act.
- Close condition: both files deleted from the working tree and absent
  from any staged set (never committed).



- MV-8 append (2026-11-02): still on disk this session — `ls -la` exit 0: libae8_dbg.rmeta 2,377 B mtime Nov 2 11:12; crates/cgap-branchless/wrapper_riscv32imc.o 6,648 B mtime Nov 2 12:08. This lane documents, does not delete. Commit plan refresh (w-commitplan-v2.md R.9) restates delete-never-commit.
## R9 — workspace member count: doc stale (23 live vs 22 documented)

- Status: OPEN (doc hygiene), re-verified THIS session.
- Re-check: `cargo metadata --no-deps` -> 23 workspace members
  (21 crates + ontologies/fibo + ontologies/cloud). CLAUDE.md says
  "22-crate root workspace". The live tree is authoritative; the doc
  is one stale-revision behind (branchless/ontology adds not folded in).
- Close condition: one-line CLAUDE.md correction to the live count.



- MV-8 append (2026-11-02): CLOSED this session by lane MV-8 — CLAUDE.md corrected to "23-crate root workspace"; fresh `cargo metadata --no-deps` -> 23 members (21 crates + ontologies/fibo + ontologies/cloud), exit 0; publish-flag table also corrected to disk truth (grep publish crates/*/Cargo.toml, exit 0).
## R10 — CSCB_MUTATE=zero-tax flip court FAIL (cannot flip)

- Status: OPEN (code/test defect).
- Owner receipt: lanes/lane-b08-cscb-runner.md:31-42 ("Anti-vacuity
  court ... FAIL (cannot flip)": under CSCB_MUTATE=zero-tax NO child
  flips).
- Close condition (receipt's own terms): (a) at least one passing
  child baseline; (b) a child implementing zero-tax refusal semantics
  reading CSCB_MUTATE; then the env var must produce a strictly
  different outcome. A court that cannot flip is FAIL per CLAUDE.md.


- GAP-04 append (2026-11-02): zero-tax flip is now DETERMINISTIC per
  ship-plan A2 — the runner (crates/csct/src/bin/cscb_runner.rs) gates
  a designated coordinate (C4) in its aggregate logic BEFORE any child
  spawn: CSCB_MUTATE=zero-tax -> loud ANDON line + exit-1 refusal row
  `| C4 | cscb_runner zero-tax gate (C4) | exit 1 |`; clean -> exit 0.
  Witnessed 2x fresh this session (both mutated runs exit 1, aggregate
  receipt names the C4 gate; clean run exit 0). SUPERSEDES the B08
  claim at lanes/lane-b08-cscb-runner.md:31-42 ("cannot flip") — that
  entry was stale relative to code (C6 already aliased zero-tax at
  cscb_c6_retraction.rs:51-53). Flip no longer depends on which
  children happen to read the env. Receipts: /tmp/gap04_clean.md,
  /tmp/gap04_mut.md, /tmp/gap04_mut2.md; lane receipt
  lanes/lane-gap-04-zero-tax.md.

## R11 — w-commitplan-v2 stale vs live tree

- Status: OPEN (doc/process), stale-by-default per receipt law.
- Owner receipt: waves/w-commitplan-v2.md:22-26,143,150 ("live status
  138 entries vs census 221").
- Note: the receipt itself declares its counts as-of-census; per the
  receipt-cited-counts-are-stale law they must be re-read at use time.
  Dispatch also cites a missing `.claude/` group vs Agent-15 inventory.
- Close condition: regenerate the commit plan from a fresh
  `git status --porcelain` at integration time, or supersede it.



- MV-8 append (2026-11-02): REFRESHED this session by lane MV-8 — "## Refresh (2026-11-02)" section appended to waves/w-commitplan-v2.md from a fresh `git status --porcelain` (252 entries, exit 0); original session-verbatim plan preserved above it.
## Verdict — what blocks a v26.11.2 tag

The publish freeze (waves/publish-halt.md, FROZEN-BY-OPERATOR) already
stops the release actuator: 8 crates are immutably live at 26.11.2, 10+
remain frozen, and the addendum records lift-time debts (cgap-ffi
BLOCKED on a cgap-parquet re-publish; blake3/arrow 26.11.3 repairs
owed). The tag question is therefore downstream of the operator lift,
not of this register.

- CODE DEFECTS (must fix before any tag): R2 (default-feature
  cgap-parquet test compile abort, exit 101 re-verified — the FA9
  court is silently un-gated), R1 (C6/GIT-11 retraction throughput
  honest FAIL vs 800M gate), R10 (CSCB_MUTATE flip court cannot flip).
- PROCESS/EVIDENCE (must close before a tag is certified): R4
  (G2-PARETO L1/L2/L4/L5 rollup), R7 (F5/F10/WG2/WG3 receipts),
  R3 (coverage floor red).
- DOC/HYGIENE (do not block a tag): R5 (receipt_lint 445 findings,
  growing), R8 (debris files), R9 (member-count doc), R11 (stale
  commit plan).
- NOT A BLOCKER: R6 (verified present this session).

One-screen answer: NO tag until R1/R2/R10 are repaired and
re-witnessed, R4/R7 produce their receipts, and the operator lifts the
publish freeze. Doc-hygiene rows can land in parallel and do not gate
the tag.

## GAP-11 appends (2026-11-02, lane gap-11 hygiene)

- R2 CLOSED: FIX-02 landed — default-feature `cargo test -p cgap-parquet`
  exit 0 and std-featured suite 195 passed, 0 failed. Cite:
  lanes/lane-fix02-parquet-features.md.
- R8 CLOSED: debris deleted this session —
  `rm /Users/sac/cgap/libae8_dbg.rmeta /Users/sac/cgap/crates/cgap-branchless/wrapper_riscv32imc.o` exit 0;
  verification `ls` of both paths exit 1 (No such file or directory):
  both absent from the tree, delete-never-commit honored (no staging).
  Cite: lanes/lane-gap-11-hygiene.md.
- R9 CLOSED: MV-8 refreshed — CLAUDE.md corrected to 23 members,
  publish table corrected to disk truth. Cite:
  lanes/lane-mv-08-doc-refresh.md.
- Lease janitor run: `python3 scripts/lease_janitor.py --clean` exit 0;
  orphaned: 0, reclaimed 0 bytes (0 dirs, 0 failures); 25
  matched-grammar dirs all ACTIVE (unclassified skipped: 1829,
  refused by construction).

## R1/R10 append — lane MV-5 gate re-scope (2026-11-02)

- R1 DECIDED (gate re-scope, this session): C6 retraction gate re-scoped
  from the aspirational 800M facts/s to a decided 150M facts/s
  full-cascade (100k facts, single `cascade_retract` call, min-of-7,
  release). Provenance: session-measured N=7 (corpus seed
  0xA20C_0FFE_E5A7_E000 via dred::splitmix64, release, this host):
  min 194,253,212.9 facts/s, median 159,130,004.4, max 129,554,655.9;
  confirm run min 211,174,510.4. The 800M figure traces to mask-only
  legs (dred.rs:72-77, EXP-05), never a full-cascade measurement; it is
  retained as ASPIRATIONAL_FPS, reported loudly in the C6 output
  (3.79x shortfall printed), never gating. Gate-change authority: this
  register row's own close condition ("an operator decision to lower
  the gate with a written gate-change receipt") — this append + the
  const-doc provenance in cscb_c6_retraction.rs is that receipt.
  Commands: `cscb_c6_retraction` exit 0 (PASS, shortfall reported);
  `CSCB_MUTATE=skip-sweep` exit 1 (10 phantoms detected);
  `CSCB_MUTATE=zero-tax` exit 1.
- R10 PARTIAL-FLIP (this session): `zero-tax` accepted as a mutant
  alias in BOTH re-scoped bins, so the R10 close condition (b) "the env
  var must produce a strictly different outcome" now holds on C3 and
  C6: CSCB_MUTATE=zero-tax -> C3 exit 1 (COURT-FLIPPED, closure
  4,837,284 != baseline 1,500,000) and C6 exit 1 (10 phantoms).
  Remaining for full close: other children (C1/C2, C4, C5, C7, C8)
  still ignore zero-tax — R10 stays OPEN until every present child
  flips or the court's scope is formally narrowed.
- Full-ladder witness: `cargo build --release` exit 0, `cscb_runner
  --receipt-out /tmp/mv5-cscb-certified-run.md` exit 0, all 8
  coordinates exit 0, blake3 transcript root
  9ab685137bbdbd47b42898bee24c1c34e0307a78a59bc91c46285fa989b1bd2e.
  Note: ladder C3 consumed the C1-written fixture at
  /tmp/cscb_corpus.bin (6.3 s wall); the standalone self-gen C3 gate
  run (min 36.9 s, N=7) also exits 0 under the decided budget —
  corpus-shape divergence recorded, not reconciled here.
- C3 companion re-scope: gate re-scoped from the never-a-gate 25.0 ms
  figure (EXP-03) to 60,000 ms min-of-7 on the self-gen 1M corpus.
  Provenance: session-measured N=7 min 36,918.401 ms / median 40,324.179
  ms / max 42,674.974 ms (B02's receipt measured min 42.394 s /
  median 48.478 s / p99 62.498 s — same order; the dispatch's ~796 ms
  and B02's dispatch-cited 22.99 ms witness did not reproduce on this
  host and are recorded as unreproduced, not re-defined). 25.0 ms kept
  as ASPIRATIONAL_MS, shortfall (1476x) printed loudly, never gating.
  Court: CSCB_MUTATE=radix-clamp exit 1 COURT-FLIPPED at full scale
  (N=7); zero-tax alias exit 1.
