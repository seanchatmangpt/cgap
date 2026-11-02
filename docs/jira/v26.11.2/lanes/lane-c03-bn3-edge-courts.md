# Lane C03 — BN3 edge/closure courts (v26.11.2)

Date: 2026-11-02. Branch: main. Repo: /Users/sac/cgap (canonical checkout).

## Files written (complete list)

- `crates/cgap-bn3/tests/edge_closure_courts.rs` (NEW, 530 lines, 5 tests)
- `docs/jira/v26.11.2/lanes/lane-c03-bn3-edge-courts.md` (this receipt)
- `crates/cgap-core/tests/`: nothing written — no core test file was
  needed (see Divergence 2).

Shared tree untouched otherwise. No git commands run.

## Real API discovered and cited (task 1)

All courts run against the real public surface, re-read this session:

- `cgap_bn3::unify::{unify_to_fixpoint, unify_batch_probe, RuleCompiler,
  Rule, BodyAtom, StatementColumns, EvalScratch, JoinProbeScratch,
  MatchSet, UnifyError, DEFAULT_CHUNK_ROWS, RADIX_BUCKETS}` (unify.rs)
- `cgap_bn3::sigil::{encode_term, Sigil::{SlotConst, VarUni}}` —
  SlotConst word == payload (sigil 0x00)
- `cgap_bn3::project::DeltaBatch { rule_id, epoch, rows: DeltaRows }`
  (read off batch.rows.{s,p,o,len})
- No retraction API exists in cgap-bn3 — retraction is modeled as
  re-running `unify_to_fixpoint` on the pruned base graph and comparing
  derived sets (see Court E4).

## The courts (task 2) and their twins (task 4)

Target rule: transitive closure `(?x E ?y),(?y E ?z) => (?x E ?z)`.

### E1 zero-edge graph — `e1_zero_edge_graph_derives_nothing_and_converges`
Empty store: `unify_to_fixpoint` returns Ok, exactly 1 batch, 0 rows,
no `FixpointBudgetExceeded`.
Twin M-E1 (in-test): add chain a->b->c — derived set MUST become
{(a,c)}; assertion flips. PASS.

### E2 self-loop — `e2_self_loop_self_joins_once_and_derives_nothing`
Store {n->n}: probe-level, the closure rule self-joins the loop row
exactly once with x==y==z==n (atom_rows[0]==atom_rows[1], same row);
fixpoint-level, no NEW derived fact.
Twin M-E2 (in-test): flip edge n->n -> m->n — match count MUST drop
1 -> 0. PASS.

### E3 saturated bucket — `e3_saturated_bucket_probe_emits_exactly_the_true_joins`
REAL bucket bound found: `RADIX_BUCKETS = 4096`
(unify.rs:840), hash `(y ^ y>>16) & 0x0FFF` on the 56-bit payload
window (unify.rs:850-854). NOT the 65,376-lane class from CLAUDE.md
(that is a RingSink LANES bound, a different surface).
Corpus: 100 seeds + hub + 150 distractor rows, every join key hashing
to bucket 0; the bucketed probe must emit exactly the 100 true joins
(probe-level out.len==100) and the closure exactly seeds->t (no
phantom joins). Replica-hash self-check included (court fails loudly
if the kernel hash changes).
Twin M-E3 (in-test): flip ONE seed edge (s1->v -> s1->w, out-of-shape)
— derived loses s1->t, count 100 -> 99. PASS.

### E4 cascading retraction — `e4_cascading_retraction_alternate_path_survival_and_cascade`
Graph G1={ab,bc,cd,de,bx,xc}, alternate derivation path b->x->c.
Differential against an independent BFS closure oracle (different
algorithm; the lever_courts/bgp_hop_courts pattern):
- A: D1 == oracle closure(G1)-G1 == 9 facts (kernel and oracle agree).
- B: retract bc alone — ZERO D1 facts lost; bc flips base->derived
  (re-derivable via x); a->e survives. Multiple derivation paths hold.
- C (twin of B): retract xc too — cascade completes: D3 == {ax, ce};
  b->c and all downstream facts gone, no phantom survivors.
All three steps assert set equality with the BFS oracle = the
no-phantom-survivors check. PASS.

### E5 zero-alloc steady state — `e5_zero_alloc_steady_state_driver`
AE7 `EnclaveTrackingAlloc` (task 3), included via `#[path]` (Divergence 1).
Single #[test] driver per the sequencing law. Windows:
- positive control: alloc under budget(0,0) MUST breach
  (witnessed: `ALLOC BUDGET BREACH: site="c03-positive-control"`);
  this is the anti-vacuity twin for the whole of E5.
- warmup: bounded poll on window counts (no sleep) until quiet;
  asserts convergence by the 10th call (observed: 3rd).
- steady state: 20x `unify_batch_probe` on the E3 corpus under
  budget(0,0) — counts asserted (0,0). PASS.
- honesty arm: `unify_to_fixpoint` is NOT zero-alloc (`seen`
  BTreeSet, AE7 note in crates/cgap/tests/alloc_boundary.rs:24-27);
  pinned as measured nonzero counts under an unbounded window,
  loudly, not faked to 0.

## Commands and exits (this session)

| command | exit | result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/c03-target cargo test -p cgap-bn3 -p cgap-core` (BEFORE additions) | 0 | bn3 119+2+7+3, core 17+8+2+11+4+6+3+2+8+4+13, 0 fail |
| `CARGO_TARGET_DIR=/tmp/c03-target cargo test -p cgap-bn3 --test edge_closure_courts` | 0 | 5 passed, 0 failed |
| `CARGO_TARGET_DIR=/tmp/c03-target cargo test -p cgap-bn3 -p cgap-core` (AFTER additions) | 0 | gate fully green; edge_closure_courts 5/5; bn3 unit 119/0; no regressions |
| sandbox `/tmp/c03-sandbox` baseline | 0 | 5 passed (sandbox = copied bn3 crate + standalone Cargo.toml) |
| sandbox M1 (fixpoint convergence check killed: `if new_rows==0` -> `if false`) | 1 | 5/5 courts FAIL — flip witnessed |
| sandbox M2 (`DEFAULT_MAX_ROUNDS` 64 -> 2) | 1 | e4 fails (FixpointBudgetExceeded) — flip witnessed |
| sandbox M3 (radix probe bound-slot join compare dropped, unify.rs:1441) | 1 | e3 fails (phantom joins) — flip witnessed |
| sandbox restore pristine + rerun | 0 | sha256 `1f2e8db8bb7738a8…d40062` byte-exact, 5/5 green |

Mutation notes: an earlier M3 attempt mutated unify.rs:1467 — that line
is the RETIRED nested-loop arm (dead under default `radix: true`); the
court did not flip, the mutation was corrected to the radix path
(:1441) and then flipped. A hang-class first M1 (seen-set dedup killed)
blew up the store exponentially and was replaced by the convergence-
check mutation above (clean typed flip via FixpointBudgetExceeded).
Shared tree was never mutated — all mutations were /tmp sandbox copies
with sha256-exact restore + mtime touch.

## Divergences (doc vs reality)

1. **tracking_alloc not public.** cgap-core src/tracking_alloc.rs exists
   but lib.rs exports only dma_ring/frame/isomorphism (lib.rs is
   off-limits to this lane). Mirrored the landed pattern
   `crates/cgap/tests/alloc_boundary.rs:50-53`: `#[path]` include into
   the test binary + binary-local `#[global_allocator]`. Compiles with
   NO Cargo dependency changes (module is core+std only).
2. **No cgap-core test file needed.** The lane dispatch allowed NEW
   files under crates/cgap-core/tests/, but every court surfaces through
   cgap-bn3's public API; nothing core-side to test that core tests
   don't already cover. Zero files written there.
3. **Retraction is not an API.** "Cascading retraction" is modeled as
   re-running the fixpoint on pruned base graphs (E4) — bn3 has no
   retraction surface; the re-derivation-survival semantics are still
   pinned against a BFS oracle.
4. **E5 warmup needs 3 calls, not 2.** Measured: the first post-warmup
   probe call allocates once (2048 B, a buffer high-water); the court
   polls window counts until quiet (bounded, no sleep) and asserts
   convergence by call 10.

## Standing

ALIVE. Every claim above backed by a command+exit run this session
(table above); 5/5 courts green on the shared tree gate AND on the
sandbox; 3/3 mutations flip; restore sha256-exact.

## Cleanup

/tmp/c03-target deleted at lane end (build-root lease law). Sandbox
/tmp/c03-sandbox deleted as well.
