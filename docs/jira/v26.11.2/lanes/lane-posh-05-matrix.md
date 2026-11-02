# Lane POSH-5 — Refusal Matrix Court (stitch index POSH-1..4) — Receipt

Date: 2026-11-02. Branch: main. Files owned/written:

- `crates/cgap-git/tests/posh_refusal_matrix_court.rs` (new, 208 lines)
- `docs/jira/v26.11.2/lanes/lane-posh-05-matrix.md` (this receipt)

lib.rs, Cargo.toml, all src/: UNTOUCHED. No git commands. Build lease
`/tmp/posh5-target` (deleted at lane end).

## 1. Identity

Subject: the four-arm refusal matrix over the REAL mounted APIs of
cgap-git — `pack_demux::Demux`, `admission_gate::{RefTable, admit_update,
run_closure}` and `ingress_ring::IngressRing<INGRESS_RING_BYTES>` —
stitching POSH-1's wire law, GIT-09's gate law, GIT-03's ring law, and a
POSH-4-tied alloc positive control into one court file.

## 2. Authority

Dispatch POSH-5 (coordinator prompt): write only the matrix court + this
receipt; flip AS the test (in-test mutants, no /tmp source mutation).
Cross-read first: lane-git-09-admission-gate.md, lane-git-03
(ingress_ring.rs capacity), lane-git-01 (FACT_WINDOW provenance).

## 3. Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/posh5-target cargo test -p cgap-git --test
  posh_refusal_matrix_court` → exit 0, **4/4 passed**.
- `CARGO_TARGET_DIR=/tmp/posh5-target cargo test -p cgap-git
  --no-fail-fast` → exit 101 overall; per-binary census: 24 test binaries
  ok; 2 OTHER lanes' binaries carry failures (see Divergences 3).
  posh_refusal_matrix_court 4/4, posh_wire_court 3/3 (POSH-1),
  posh_alloc_sentry_court 1/1 (POSH-4) all witnessed in this run.
- `CARGO_TARGET_DIR=/tmp/posh5-target cargo test -p csct --test
  posh_etag_court --manifest-path /Users/sac/cgap/crates/csct/Cargo.toml`
  → exit 0, **5/5 passed** (POSH-3 row re-verified, not receipt-cited).
## 4. Courts — the 4x2 matrix (each arm: BEFORE clean / AFTER mutant)

| Arm | Mutation (flip AS the test) | BEFORE (clean) | AFTER (typed refusal) |
|---|---|---|---|
| 1 bad magic | `ARM\0` -> `XRM\0` in-test (POSH-1 wire law) | clean pack (real `write_pack_v2`) demuxes: `finish()==Ok`, 1 object | `DemuxError::SubjectMissing` (gate exit-2 class); 0 objects, stream never Done |
| 2 cyclic injection | A->B->A 200-node ring + transitive rule through the REAL `RefTable::apply` | clean ground facts admit; ref advances 0x11->0x22 | `GateError::CyclicDerivation { rounds, facts>4096 }` (AndonRefusal class); `table.bytes()` byte-identical before/after |
| 3 stride overflow | fact 1,366 pushed at the ring's real window | 1,365 facts fill 32,760 B, pop drains byte-exact | `IngressError::Overflow { head-tail=32760, frame_len=24 }`, `andon_exit_class()==101`; head/tail untouched |
| 4 alloc control | forced heap alloc (`Vec::with_capacity(64)`) | sentry baseline b | sentry observes b' > b — dead instrument FAILs, never skips |

Arm-3 bound is DERIVED, not magic:
`MAX_INLINE_FACTS = INGRESS_RING_BYTES / FACT_STRIDE = 32768/24 = 1,365`;
1,366 x 24 = 32,784 > 32,768 refuses. Matches GIT-01's named
`FACT_WINDOW = 1_365` (pack_demux.rs:43-51 provenance note).

## 5. Standing: ALIVE

For the exact subject `posh_refusal_matrix_court.rs`: 4/4 green, exit 0,
this session, against the real mounted lib APIs (no #[path] include
needed — lib.rs already mounts all modules).

## Stitch index POSH-1..4 (re-read from disk at receipt time)

| Peer | Receipt | Status | Evidence |
|---|---|---|---|
| POSH-1 wire | lane-posh-01-wire.md | ALIVE | posh_wire_court 3/3 this session (census run) |
| POSH-2 | lane-posh-02-*.md | **PENDING** | no receipt, no posh-02 test file on disk — per dispatch, not waited on, not created |
| POSH-3 etag | lane-posh-03-etag.md | ALIVE | csct posh_etag_court 5/5 exit 0 this session |
| POSH-4 sentry | lane-posh-04-sentry.md | ALIVE | posh_alloc_sentry_court 1/1 this session (census run) |
| POSH-5 matrix | this receipt | ALIVE | 4/4 exit 0 this session |

## 6. Must-not-regress fences (state-and-run law)

Fences named: admission_gate_court (6/6), pkt_line_court (10/10),
pack_demux_court (10/10), ingress_ring_court (6/6) — all re-run green in
the no-fail-fast census above. My diff is ONE new test binary; its
`#[global_allocator]` is process-local to that binary and cannot affect
any other test binary.

## 7. Divergences (doc yields to reality)

1. Dispatch named the arm-3 refusal "GateFail-class". No `GateFail`
   exists in the codebase; the real typed refusal is
   `IngressError::Overflow` with `andon_exit_class() == 101` (the same
   101 family the dispatch calls AndonRefusal). Court asserts the real
   variant.
2. Dispatch's "magic ARM\0" is POSH-1's TEST-LOCAL `WireChunkHeader`
   law (posh_wire_court.rs), not a lib type; the lib-side subject-missing
   law is the demux "PACK" check. Arm 1 corrupts ARM\0->XRM\0 (POSH-1
   flavor) and corroborates through the real demux law, same as POSH-1's
   own court 1.
3. PRE-EXISTING failures in OTHER lanes' binaries (compile-abort/run-abort
   = not a failure count of this lane): octopus_merge_court
   `mutation_conflict_injection_flip` (1 fail) and e2e_wire_substrate_court
   `wire_to_substrate_e2e` (1 fail). This lane owns neither file; a
   single-file test-binary diff cannot flip them.
4. POSH-4's instrument HAD landed at implementation time
   (posh_alloc_sentry_court.rs, 1/1 green). The dispatch's
   implement-if-absent clause did not trigger; this file still carries a
   LOCAL counting-alloc control because allocator state is process-global
   per test binary — POSH-4's sentry is unobservable from this binary.
   Disclosed here; no cross-lane edit was made, so no both-directions
   disclosure is owed.

## 8. Cleanup

`/tmp/posh5-target` deleted via python shutil.rmtree at lane end. Shared
tree carries exactly the two owned files as changes.
