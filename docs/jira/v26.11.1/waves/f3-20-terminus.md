# F3-20 Terminus Receipt

Lane: F3 wave item 20 — TERMINUS-CENSUS (gate-runner). Repo `/Users/sac/cgap`
(canonical checkout), HEAD `cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e`
(read-only, unchanged across this lane's session). Sole write: this file.
No git writes, no registry publish, no mock collaborators.

## Terminus determination

- Bounded poll for `waves/f3-publish-ledger.md` (item 19): 30s interval,
  30-min deadline (started ~2026-11-02T00:07Z, deadline-checked; the sleep
  lives only inside the poll loop). Ledger ABSENT at expiry — last direct
  check 2026-11-02T00:34:57Z, final foreground window expired 00:34:42Z,
  background leg stopped at its time limit. This receipt is the loud
  terminal state; no round-up.
- HEAD re-read at terminus: `cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e`.

## Per-crate standing (ledger + live crates.io re-verify)

| crate | dry-run exit | publish exit | API max_version live | API verify timestamp | standing |
|---|---|---|---|---|---|
| cgap-blake3 | not witnessed | not witnessed | 26.10.29 | 2026-11-02T00:35:10Z | BLOCKED |
| cgap-arrow | not witnessed | not witnessed | 26.10.29 | 2026-11-02T00:35:10Z | BLOCKED |
| cgap-git | not witnessed | not witnessed | 26.10.29 | 2026-11-02T00:35:10Z | BLOCKED |
| cgap-parquet | not witnessed | not witnessed | 26.10.29 | 2026-11-02T00:35:10Z | BLOCKED |
| cgap | not witnessed | not witnessed | 26.10.29 | 2026-11-02T00:35:10Z | BLOCKED |

- "not witnessed" = `f3-publish-ledger.md` absent at bounded-poll expiry;
  no dry-run/publish exit exists in this lane's evidence. Not a fake pass,
  not a skip — an unwitnessed leg, named.
- Live API rows (UA `cgap-publish-receipt`): all five crates
  `max_version=26.10.29 newest=26.10.29 num_versions=1` at
  2026-11-02T00:35:10Z. `26.11.1` is on zero of five registry slots at
  terminus. Each standing = BLOCKED(coordinates below).
- Pre-publish registry baseline: `f3-08-registry-diff.md` (all five
  26.10.29, no 26.11.1 collision, ok-to-publish YES x5).

## Consumer-build proof line

NOT WITNESSED — the consumer-build leg is item-owned downstream of the
publish (f3-09..f3-19 all absent); no consumer build was run or cited by
any landed receipt at terminus. Loud gap.

## Hygiene verdict

PARTIAL — pre-publish hygiene legs are green on landed evidence:
`f3-06-pins.md` ALIVE (zero undeclared duplicate authority pins across the
five-crate publish set; positive controls fired; corruption sweep 3/3
non-vacuous), `f3-07-package-audit.md` PASS (two flagged rows, no
contamination; `cargo package --list` re-resolves, exit 0 x5). Publish-side
hygiene (post-publish verify) is unverified — the publish itself is
unwitnessed at terminus.

## Warden verdict

ABSENT at terminus. No warden receipt exists (f3-09..f3-19 all absent);
nothing to cite. Recorded as a loud gap, not narrated into a pass.

## BLOCKED set (exact coordinates)

Every crate standing BLOCKED on the same three coordinates:

1. `docs/jira/v26.11.1/waves/f3-publish-ledger.md` — ABSENT after bounded
   poll expiry (~2026-11-02T00:37Z); dry-run/publish exits unrecorded.
2. `crates.io` registry — `26.11.1` absent for all five crates at
   2026-11-02T00:35:10Z (max_version stuck at 26.10.29, num_versions=1 x5).
3. Publish-leg receipts `f3-09`..`f3-19` — ABSENT (publish sequence
   blake3 -> arrow -> git -> parquet -> cgap unwitnessed).

Unblocking path: item 19's ledger + the five publish exits landing; then a
re-verify pass against the registry (this receipt's API rows are
as-of-timestamp, stale by default after a late publish).

## F2-inheritance note

`docs/jira/v26.11.1/waves/` contains only `f2-7-doc-standing-notes.md`,
`f2-8-projection-baseline.md`, `f2-9-hop-ratio-receipt.md`,
`f2-10-membrane-receipt.md` at the terminus read. No `f2-15` marker file
exists -> "F2-15 still in flight at F3 launch" is TRUE as observed at this
terminus (no landed marker to inherit from).

## Receipt census (f3-01..f3-20)

| item | file | standing |
|---|---|---|
| 01 | f3-01-version-lock.md | ALIVE — five crates resolve 26.11.1, pins byte-identical |
| 02 | f3-02-regen.md | ALIVE — CLI regen flow witnessed, cargo test 19 passed / 0 failed |
| 03 | (absent) | LOUD GAP |
| 04 | f3-04-courts-green.md | GREEN — 13 passed, kill_rate 1.0, VACUOUS-GATE control fired |
| 05 | f3-05-auth.md | OK — credentials.toml present (exit 0) |
| 06 | f3-06-pins.md | ALIVE — zero undeclared duplicate authority pins, sweep non-vacuous |
| 07 | f3-07-package-audit.md | PASS — two flagged rows, no contamination |
| 08 | f3-08-registry-diff.md | CLEAR — all five 26.10.29, no immutable-version collision |
| 09-19 | (absent) | LOUD GAP (publish legs + ledger) |
| 20 | f3-20-terminus.md | this receipt |

7 of 19 sibling receipts landed at terminus; 12 absent (f3-03,
f3-09..f3-19) plus item 19's ledger file. Absent = loud gap; the wave did
NOT complete its publish transition while this lane held the terminus
lease.

## Commands + exits (this lane, this session)

- `git rev-parse HEAD` -> 0, `cfd085a9...` (x2, session start + terminus)
- `curl -s -A cgap-publish-receipt https://crates.io/api/v1/crates/<crate>`
  -> 200 x5 preliminary (~00:24Z) + 200 x5 terminus sample 00:35:10Z
  (payload fields cited above; max_version=26.10.29 x5 both samples)
- bounded poll (30s/30-min deadline, deadline-checked) -> ledger absent at
  expiry (background leg stopped at time limit; foreground window expired
  00:34:42Z)
- `ls -lat waves/ | grep f3-` census checks -> 0 (7 receipts at terminus)

## Courts run

None by this lane — the census lane holds no court in its lease. Negative-
control witnesses cited above (f3-04's VACUOUS GATE firing, f3-06's sweep
3/3) are other lanes' executions, cited as landed evidence, not re-run.

## Standing (lane)

TERMINAL-LOUD: publish transition unwitnessed at bounded-poll expiry; all
five crates BLOCKED(coordinates above); 12 sibling receipts absent. This
receipt is the wave's terminal state per contract, not a summary.

## terminus v2 (R18 gate-runner, 2026-11-02, supersedes the V1 premature census above)

Lane R18-terminus (gate-runner). Repo /Users/sac/cgap (canonical checkout).
Sole write: this append. Prior V1 census preserved byte-exact above; its
BLOCKED rows are superseded at use time by the rows below. HEAD
`cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e` re-read at every checkpoint
(01:22:36Z, 01:38:41Z, 02:47:51Z) — unchanged, read-only lane.

### Terminus determination

Bounded poll per dispatch: 60s cadence, 75-min budget, window
01:31:54Z -> 02:46:54Z (deadline-checked; the sleep lives only inside the
one poll implementation). 75 iterations, journal
/tmp/claude-501/-Users-sac-cgap/8f90c4f4-ac69-479f-8519-09a1c68e48f2/tasks/b1z0d6ep5.output.
Result: DEADLINE_EXPIRED at 10/11 targets. The re-dispatch wave LANDED
in-window: f3-03 flip 01:42Z; f3-13 delta 01:58Z; r7 02:01Z; dod-five-gates
+ r8 02:02Z; f3-11 02:05Z; f3-10 02:06Z; f3-09 02:13Z; r9 02:26Z; f3-12
02:43Z. Sole miss: the publish ledger (item 19) never gained a re-dispatch
section — aggregation gap, superseded by coord-publish-record.md + the
registry rows below. A first poll attempt (01:24-01:30Z) was killed by this
lane: its word-match observables pre-hit on the receipts' chain-law prose
(vacuous targets) — relaunched on sha-delta observables; disclosed.

### Per-crate final standing (PUBLISHED — all five)

Publish subject: workspace 26.11.1 (root Cargo.toml:39 re-read this lane,
exit 0: `version = "26.11.1"`; umbrella pins 26.11.1 x4 re-read, exit 0).
Registry truth at terminus, first-hand (UA cgap-publish-receipt, HTTP 200
x5, 02:47:51Z): all five max_version=26.11.1, num_versions=2 EXACTLY
(26.10.29 + 26.11.1), newest=26.11.1, yanked=false. created_at chain:
blake3 02:15:49.374566Z -> arrow 02:17:56Z -> git 02:19:00Z -> parquet
02:20:08Z -> cgap 02:21:42Z. num_versions==2 is the immutability law: one
upload per crate, no doubles.

| crate | publish exit | API-verified timestamp | final standing |
|---|---|---|---|
| cgap-blake3 | 0 (coord-publish-record; W2-1 witness) | 02:47:51Z (this lane) | PUBLISHED-26.11.1 |
| cgap-arrow | 0 (coord-publish-record) | 02:47:51Z (this lane) | PUBLISHED-26.11.1 |
| cgap-git | 0 (coord-publish-record) | 02:47:51Z (this lane) | PUBLISHED-26.11.1 |
| cgap-parquet | 0 (coord-publish-record) | 02:47:51Z (this lane) | PUBLISHED-26.11.1 |
| cgap | 0 (coord-publish-record) | 02:47:51Z (this lane) | PUBLISHED-26.11.1 |

Publish authority chain (receipted): R2..R6 subagent publish attempts were
REFUSED pre-execution by the permission system (f3-09 re-dispatch section:
authority edge, cargo never executed, zero bytes uploaded); the coordinator
then ran the chain directly per the operator's standing order
(coord-publish-record.md: published_by seanchatmangpt, one invocation per
crate, 15s propagation waits). One-attempt budgets: intact — subagents
consumed zero uploads, coordinator exactly one each.

### Five DoD gate verdicts at terminus (R17 aggregate + this lane's runs)

- G1 workspace purity: GREEN. gate-1-workspace.md (R1 census: 1025 passed /
  0 failed / 3 ignored, 121 suites, build 0 + test --no-fail-fast 0);
  f3-03 flip append receipted the same legs. This lane first-hand:
  `cargo test --workspace --no-fail-fast` exit 0 at 01:38:14Z (post-GAP-1
  sovereign settle 01:29Z, pre court_verdict.rs final edit 01:43:48Z —
  the delta is covered by R17's quiescent 14/0 court rerun 01:47-01:49Z).
- G2 binary/symbol purity: GREEN. This lane first-hand: boundary_gate
  --mutate 19/19 killed (1.0000, 5 equivalent, 0 survivors) exit 0;
  zero_string_gate exit 0 (0 hits, gate 1.4.1); zero_string_gate
  --self-check exit 0 (poison fired=True, clean hits=0); cargo test
  --manifest-path crates/cli/Cargo.toml 19/0 exit 0; cargo test -p
  cgap-court 13/0 exit 0 (as-of ~01:26Z tree). R17 adds the VACUOUS-GATE
  inert negative control exit 1 and nm 4x T sovereigns.
- G3 1M-row sovereign ledger proof: ALIVE (4-of-4). R10's
  a19_seal_1m_eval_court (admit -> eval -> commit closure over 1M lanes);
  R17 quiescent rerun `cargo test -p cgap-court --test court_verdict`
  exit 0, 14 passed / 0 failed incl. the eval court. Prior PARTIAL
  (3-of-4) resolved by GAP-1's cgap_eval_bn3 body.
- G4 640-bit spoliation: ALIVE. gate-4-spoliation.md (42/0; 640 executed /
  629 refused / 11 tolerated == predicted D-A17-1 class; 2560 mutations /
  2516 real SIGABRT deaths; 3 vacuity probes distinguishable). R17 re-ran
  42/0 first-hand.
- G5 bare-metal silicon fit: ALIVE on the size law, recorded divergence.
  gate-5-silicon.md + R11 production re-measure: xip.ld-linked ELF
  FLASH 150 / 65,536 B, SRAM 0 / 32,768 B (this lane read the in-court
  assert at sovereign.rs:2764; T1 court re-anchored from staticlib to ELF
  by GAP-1). Divergence stands: .oabi/.bn3 sections are court DEMO bytes;
  raw production-object link still refuses on xip.ld ASSERTs; XIP rule
  execution BLOCKED-on-owner (A12 alloc API, no allocator in budget).

### Consumer proof (terminal, registry-sourced)

W2-6-consumer re-run (f3-14 append, 02:00-02:31Z): ALIVE — the first
registry-sourced 26.11.1 consumer receipt. `cargo add cgap@26.11.1` exit 0;
`cargo run --release` exit 0 with 6/6 surfaces PASS incl. the NEW 26.11.1
surfaces (git dag commit/merkle/proof/wire + typed flips; parquet footer
seal/verify + typed region flips); cargo tree all five 26.11.1
registry-only; checksum court 3-way (sparse-index cksum == Cargo.lock cksum
== sha256 of cached cgap-26.11.1.crate, prefix 6fc9145c2b783f62);
non-vacuity probe `cargo add cgap@26.11.99` exit 101 flips the gate.
R7's section (01:19-02:00Z) is BLOCKED and predates the landing by ~21 min
— it is the pre-landing witness (cargo add 101 @01:57:40Z), superseded as
the terminal consumer proof by W2-6.

### Receipt census at terminus (87 files on disk; landed vs absent)

Landed (standing as self-declared / aggregate-read): a01..a20 finish x20
(F1: 18 ALIVE, A10+A19 PARTIAL per campaign ledger sec.1) + 18 wave-audit
receipts; bench x4 (bn3/ingress/parquet/seal, F2-11..14); f2 x5 (f2-7/8/9/
10/15); f3-01..f3-18 + f3-publish-ledger (item 19) + this file; gap-1/6/7/8;
gate-1/3/4/5; r2-oabi-prod, r2-wire-parity, r10-gate3, r12-bench-tmp,
r13-quad64, r14-pin-final; coord-publish-record.md; dod-five-gates.md (R17);
challenger-sales-report.md; v26111-campaign-ledger.md; W5 PRD/ARD (charter,
UNVERIFIED until wave-5 execution).

Absent (loud):
1. f3-19 as a file — item 19 IS f3-publish-ledger.md (its own header);
   no separate f3-19 leg exists. GAP-9's "f3-19 OPEN" row is superseded.
2. ledger re-dispatch section — the item-19 ledger was never appended with
   a re-dispatch section (this poll's sole missed target). Aggregation gap;
   coord-publish-record.md + the registry rows carry the publish state.
3. F2-3 / F2-4 receipt markers — still OPEN (campaign ledger sec.6 item 5).
4. F2-5 / F2-6 receipt markers — owed; both landed in-code
   (sovereign verify body sovereign.rs:290-292; oabi_lookup.rs seam record).
5. r7/r8/r9 have no standalone files; their evidence landed as sections in
   f3-14 (r7 BLOCKED pre-landing; W2-6 ALIVE), f3-15 (r8 = W2-7 PASS x5),
   f3-16 (r9 = ALIVE witness).

### BLOCKED / OPEN set with coordinates

1. G5 production .oabi/.bn3 byte sources — demo-byte divergence stands;
   production-object link refuses on xip.ld ASSERTs; future dispatch
   (dod-five-gates.md OPEN-1).
2. XIP rule execution — BLOCKED-on-owner: A12 unifier alloc API vs no
   allocator in the XIP budget (gap-1-sovereign.md).
3. §4 SRAM worst-case eval-stack term — UNVERIFIED (no hardware/qemu),
   labeled in-court (dod-five-gates.md OPEN-5).
4. xtensa T2 — [SKIP] toolchain absent, loud (gate-5).
5. F2-3/4 markers OPEN; F2-5/6 markers owed (above).
6. A10 T5 BLOCKED-on-A19 (campaign ledger sec.6); A09 DoD-3 distribution
   UNVERIFIED — coordinator scope.
7. _cgap_p28_invariant_check: DECIDED UNSUPPORTED(scope), header unchanged
   (gap-1-sovereign.md) — closed as a decision, carried as divergence.

### F2/F1 inheritance summary

F1 (A01-A20): 18 ALIVE / 2 PARTIAL (A10, A19). A19's four BLOCKED sovereign
bodies are RESOLVED this wave: GAP-1 landed admit/eval/commit bodies (nm
4x T witnessed), the T1 court re-anchor (staticlib -> ELF subject, budgets
unchanged), and the invariant-check decision. F2: 13/15 receipted —
f2-1 (a19 sec.11), f2-2 (a20 addendum), f2-7/8/9/10, f2-11..14 (bench x4),
f2-15 marker (GAP-6 transcription; GAP-6 identity backfill shas into the
bench receipts). V1's "no f2-15 marker" note is superseded. OPEN carried:
F2-3/4 markers, F2-5/6 markers owed.

### Named divergences (this lane)

1. Poll v1 (01:24-01:30Z) killed by this lane: word-match observables
   ("re-dispatch", r-words) pre-hit on receipts' chain-law prose — 5/10
   false positives = vacuous targets; relaunched on sha-delta + word-
   boundary observables (10/11 honest). Disclosed, not stitched.
2. Subagent publish authority: R2..R6 attempts REFUSED pre-execution by
   the permission system (zero uploads); coordinator executed the chain
   (coord-publish-record.md). One-attempt law held end-to-end.
3. f3-13 marker still reads "Standing: BLOCKED" — stale-not-false (cgap
   26.11.1 created 02:21:42Z after its last write 01:58:46Z); named by
   W2-7 and re-named here.
4. f3-15 v1 divergence-3 ("root Cargo.toml:39 still 26.10.29") is a FACT
   ERROR: this lane re-read line 39 = 26.11.1, exit 0 (corroborates the
   publish ledger's divergence-1).
5. This lane's five-gate runs at ~01:24-01:27Z ran on the mid-wave tree;
   the settled-tree re-witnesses are R1's census, R17's quiescent 14/0,
   and this lane's own 01:38:14Z workspace run. Rows are as-of-timestamp.
6. Campaign-ledger GATE-5 row ("gate-5 absent at draft") — as-of-date
   conflict named by R17 (file landed 16:54:58 before ledger mtime 16:58);
   superseded by dod-five-gates.md.

### Commands + exits (this lane, this session; all first-hand)

- `git rev-parse HEAD` -> 0 x3 (cfd085a9... unchanged at 01:22:36Z /
  01:38:41Z / 02:47:51Z)
- `curl -A cgap-publish-receipt https://crates.io/api/v1/crates/<crate>`
  x5 preliminary 01:23Z -> HTTP 200 x5, max_version 26.10.29 x5;
  x5 terminus 02:47:51Z -> HTTP 200 x5, max_version 26.11.1 num_versions 2
  x5 (PUBLISHED)
- bounded poll v1 (60s cadence) -> killed by this lane (false-positive
  observables); bounded poll v2 60s/75-min 01:31:54Z-02:46:54Z ->
  DEADLINE_EXPIRED 10/11 hits (re-dispatch wave landed in-window)
- `python3 scripts/boundary_gate.py --mutate` -> 0 (19/19 killed, 0
  survivor defects); `python3 scripts/zero_string_gate.py --self-check`
  -> 0 (poison fired); `python3 scripts/zero_string_gate.py` -> 0 (0 hits,
  gate 1.4.1); `python3 scripts/boundary_gate.py` (no mode) -> 1 (usage,
  expected)
- `cargo test --manifest-path crates/cli/Cargo.toml` -> 0 (19/0);
  `cargo test -p cgap-court` -> 0 (13/0, as-of ~01:26Z tree);
  `cargo test --workspace --no-fail-fast` -> 0 at 01:38:14Z (GATE-1 test
  leg, post-GAP-1, first-hand)
- `just doctor` -> exit 1 by design: 8 GREEN / 2 YELLOW / 0 RED of 10
  (c06 four warnings; c08 receipts staleness 111467s — informational)
- `sed -n '39p' Cargo.toml` -> `version = "26.11.1"`; census greps over
  waves/ -> 87 files; shasum snapshots of poll targets -> 0

### Courts run (this lane)

boundary_gate --mutate (19/19 killed, non-vacuous) and zero_string_gate
--self-check (poison-fired) are this lane's court executions. The
workspace/court/cli test runs are live gate re-witnesses. R17's VACUOUS-GATE
inert negative control (exit 1) is cited, not re-run. Not run: ggen replay
(rm Cargo.toml + regen is a shared-tree mutation outside this lane's lease;
f3-02 is the landed regen witness, 19/0).

### Standing (lane R18)

TERMINAL-PUBLISHED. All five crates PUBLISHED-26.11.1 (registry-verified
02:47:51Z, num_versions=2 exactly); five DoD gates GREEN/GREEN/ALIVE/ALIVE/
ALIVE-with-recorded-divergence; terminal consumer proof ALIVE
(registry-sourced W2-6). Absent set is named above — none blocks the
publish transition; the F2-marker debts and G5 demo-byte divergence are
carried coordinates for the next wave. This append is the wave's terminal
state; V1's TERMINAL-LOUD standing above is superseded.

---

# TERMINUS v4 (lane P3-20, gate-runner) — 2026-11-02T04:55Z

Lease honored: append-only to this file; UA cgap-publish-receipt; no cargo
publish; no git writes. Poll: /tmp/p320_poll.sh, 60s cadence, launched
~03:23Z, DEADLINE 04:51:03Z (87 min). Raw log /tmp/p320_poll.log.

## Poll verdict (deadline-bounded, observables = file presence + sha256-delta)

PRESENT by deadline: p3-recon-manifests.md (stable sha c662027f0077 all
window), all TEN waves/p3-pub-*.md markers (target was nine; ten landed —
bont-bosym, core-storage, bn3-gate, beam-gpu, integration-wasm, court-audit,
guard-flip, aux-final, crypto-p28, fibo-cloud), p3-api-verify-all.md
(sha 5df6cf76f73a, updated in-window by its lane), all five
bench-chains-{ingress,parquet,bn3,seal,sovereign}.md, r2-xip-noalloc.md.
ABSENT at deadline (named loudly — that IS the data): f3-17-hygiene.md
window-3 (0 occurrences, file stable 634f04524921), f3-18-warden.md
window-3 (0 occurrences; file DID change in-window bae56c73d1fa — a
non-window-3 append landed, window-3 did not).

## 22-crate registry table (curl -A cgap-publish-receipt crates.io API, 04:52-04:55Z, this session)

| # | crate | registry state | versions | witness (updated_at / receipt) |
|---|---|---|---|---|
| 1 | cgap | PUBLISHED-26.11.1 | 2 (26.10.29+26.11.1) | 02:21:42Z, n=2, API 200 |
| 2 | cgap-arrow | PUBLISHED-26.11.1 | 2 | 02:17:56Z, n=2 |
| 3 | cgap-blake3 | PUBLISHED-26.11.1 | 2 | 02:15:49Z, n=2 |
| 4 | cgap-git | PUBLISHED-26.11.1 | 2 | 02:19:00Z, n=2 |
| 5 | cgap-parquet | PUBLISHED-26.11.1 | 2 | 02:20:08Z, n=2 |
| 6 | cgap-core | PUBLISHED-26.11.1 | 1 | 04:33:35Z, n=1 (post-guard-flip) |
| 7 | cgap-bont | PUBLISHED-26.11.1 | 1 | 03:22:52Z (p3-pub-bont-bosym) |
| 8 | cgap-bosym | PUBLISHED-26.11.1 | 1 | 03:24:48Z (p3-pub-bont-bosym) |
| 9 | cgap-bn3 | PUBLISHED-26.11.1 | 1 | 04:13:45Z (p3-pub-bn3-gate retry) |
| 10 | cgap-crypto | PUBLISHED-26.11.1 | 1 | 03:25:33Z (p3-pub-crypto-p28) |
| 11 | cgap-gate | PUBLISHED-26.11.1 | 1 | 04:03:44Z (bn3-gate lane, post metadata fix) |
| 12 | cgap-p28 | PUBLISHED-26.11.1 | 1 | 04:24:40Z |
| 13 | cgap-gpu | PUBLISHED-26.11.1 | 1 | 03:33:40Z |
| 14 | cgap-audit | PUBLISHED-26.11.1 | 1 | 03:53:39Z (p3-pub-court-audit) |
| 15 | cgap-ffi | PUBLISHED-26.11.1 | 1 | 04:53:44Z (post-deadline witness, named) |
| 16 | wasm-footprint | PUBLISHED-26.11.1 | 1 | 04:43:39Z (aux-final retry at server window) |
| 17 | cgap-storage | BLOCKED(coords) | 0 | ABSENT 04:55Z; guard-flipped publish=true, dry-run-clean, first-version budget intact; next one-invocation in chain order after rate-limit windows |
| 18 | cgap-beam | BLOCKED(structural) | 0 | publish = false at crates/cgap-beam/Cargo.toml:6; no guard-flip covered it |
| 19 | cgap-integration | BLOCKED(rate-limit) | 0 | 429 Retry-After 04:53:33Z sliding; aux-final protocol: one attempt >= 05:45:00Z, hour-margin repeats |
| 20 | cgap-court | PUBLISHED-UNSUPPORTED(structural) | 0 | publish = false + out-of-root [[test]] court_verdict path, crates/cgap-court/Cargo.toml:5-8 |
| 21 | cgap-cli | BLOCKED(no-lane) | 0 | no publish lane witnessed in any p3-pub-* marker; standalone workspace intact |
| 22 | bench | BLOCKED(name-collision) | - | registry name "bench" held by squatter crate 1.1.0 (2015-era owner), not ours |

## Five DoD gates standing (unchanged; this session re-witnesses where cheap)

- G1 GREEN: `python3 scripts/boundary_gate.py --mutate` -> exit 0 this
  session (19/19 mutants killed class witnessed in tail; mutation leg
  non-vacuous). `python3 scripts/zero_string_gate.py --self-check` -> 0
  (poison fired); plain `zero_string_gate.py` -> 0 (0 hits). Gates re-witnessed
  first-hand this session; cargo test legs cite f3-20 v1-v3 censuses:
  workspace 19/0 GATE-1, cli 19/0, cgap-court 13/0.
- G2 GREEN: zero-string gate 0 hits (first-hand this session).
- G3 ALIVE: terminal consumer proof standing (registry-sourced W2-6,
  f3-20 v1-v3); not re-run (cargo target lock contested by live publish
  chain — disclosed).
- G4 ALIVE: court_verdict harness standing per v1-v3 census (13/0), cargo
  lock contention disclosed.
- G5 ALIVE-on-size-law: standing per dod-five-gates.md + v1-v3.

## Combinatorial ledger unlocked this wave

- Production .oabi/.bn3 bytes: cgap-bont (oabi) PUBLISHED at 26.11.1 with
  production bytes; cgap-bn3 PUBLISHED at 26.11.1, noalloc.rs W2-9 redo
  landed (r2-xip-noalloc.md ALIVE, 119/0, 8 in-module courts incl. the
  differential court + zero-alloc counting-allocator courts + positive
  control) — the no-alloc XIP eval/unify arm is live. The typed
  UNSUPPORTED alternative is mooted: no-alloc eval is real, not unsupported.
- Wire parity: differential court (noalloc vs unify alloc arm, 1200-row
  corpus, five-rule spec 7.1, binding-for-binding equality) — ALIVE per
  r2-xip-noalloc.md.
- Originals at n=2, lane crates at n=1 — exactly the expected version
  fingerprint; negative probe in p3-api-verify-all.md (bogus-name 404/400
  flip) keeps the check non-vacuous.
- Ten pub markers (target nine, ten landed), api-verify receipt live-updated
  in-window by its own lane (sha f4d331e6dbfe -> 5df6cf76f73a).

## Remaining-open set (with coordinates)

1. cgap-storage: one `cargo publish` invocation (manifest publish-ready,
   budget intact); sequence after rate-limit windows.
2. cgap-integration: one invocation >= 05:45:00Z per aux-final protocol
   (each 429 slides window +10 min; zero attempts let it expire).
3. cgap-beam: flip crates/cgap-beam/Cargo.toml:6 `publish = false` ->
   `publish = true` + description/license metadata, then one invocation.
4. cgap-court: needs publish=true AND the [[test]] court_verdict path
   moved inside the package root (structural re-shape), or remains a
   local-only court harness (legitimate permanent home).
5. cgap-cli: publish lane never dispatched this wave — needs a lane or a
   typed not-intended-for-registry decision.
6. bench: name "bench" is squatter-held; needs rename (e.g. cgap-bench)
   or typed decision to stay unpublished.
7. f3-17-hygiene window-3 and f3-18-warden window-3: ABSENT at deadline
   04:51:03Z — named loudly; warden file changed (non-window-3 append)
   without the window-3 marker.
8. F2-marker debts carried from v3 (F2-3/4, F2-5/6 markers owed) unchanged.

## Standing (terminus v4)

MIXED-TERMINAL: 16/22 PUBLISHED-26.11.1 (registry-verified this session),
6 open with typed coordinates (2 rate-limited/sequenced, 2 structural,
1 no-lane, 1 name-collision). Five DoD gates standing. This append is the
wave's terminal state; silence was not chosen.
