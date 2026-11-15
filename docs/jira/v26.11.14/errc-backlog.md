# v26.11.14 ERRC backlog (C3 synthesis, updated after fix lanes F1-F9)

Folded from the 18 lane receipts in `lanes/` plus the four crate-scoped
evals (`errc-eval-{ad,el,mr,sz}.md`). Every row cites its lane receipt; no
row is invented. Ranking: closes a standing divergence or false claim first,
then risk-per-effort. Standing words are the lanes' own.

## Landed this sweep

| Lane | Landed | Standing |
|---|---|---|
| E2 | `kd7_p28_header_court` (3 tests, 4 mutations flip) | PARTIAL |
| E4 | OI-1 closed (premise stale; bin already declared) | ALIVE |
| E6 | parquet `size_i32` x5 -> 1; git `adler32` re-export | ALIVE |
| R3 | `chains-parquet` `encode_into_arm`: 0 allocs vs 1 + 13/17 grows | PARTIAL |
| R5 | 4 panic sites -> typed errors (core/storage) + 2 courts | PARTIAL |
| Ra1 | e2e admission-band self-test (band 0.20-0.60 empirical) | PARTIAL |
| Ra4 | 3 courts gained flipping mutations; census 43/236 lack one | ALIVE |
| Ra5 | `cgap-aoq` `repository.workspace = true` | BLOCKED |
| C1 | docs-diataxis-pack artifacts (reference model only) | PARTIAL |
| C2 | four crate-scoped ERRC evals | PARTIAL |

## Ranked open items

1. **Stale CLAUDE.md claims** (E5, E3, Ra5). Version says 26.11.3, all 26
   members are 26.11.5; `cgap-beam` is an undocumented production consumer
   of `wire-shm`; `cgap-receipt`, `journal-export`, `wasm-footprint` missing
   from the non-member list; the registry-census line predates 26.11.5.
   Close: one doc-only lane edits CLAUDE.md (needs owner approval).
2. **KD-7 register is wrong** (E2). Symbol is `cgap_p28_invariant_check`,
   already declared at `include/cgap.h:214`. `sovereign.rs:362-365` and
   `1135-1158` still say "undeclared by design". Close: fix both comments,
   correct the KD-7 row.
3. **`closure.rs:230` stack overflow** (R5). `closure_court
   court2_bitplane_parity` overflows in debug; reproduced on HEAD. Close: use
   `try_new_boxed` there, re-run the storage suite.
4. **Publish chain blocked** (Ra5). `cgap-aoq` needs `cgap-crypto` and
   likely `cgap-audit` at 26.11.5 on the registry first. Publishing stays
   frozen; sequencing is a release decision. `readme`/`keywords`/
   `categories` still absent.
5. **Alloc columns vacuous in ingress and seal** (R3). Install
   `AllocProfiler` in `chains-ingress.rs` and `chains-seal.rs` now that Ra2
   has finished. Biggest alloc lever: `cgap_eval_bn3`
   (`sovereign.rs:588-622`), ~33 allocs/frame, ABI-visible so owner lane.
6. **KD-3 twin** (E1, C2). `ATTESTATION_LEN` twins at `cgap-core frame.rs:44`,
   `cgap-p28 attestation.rs:60`, `cgap-gate penstock.rs:239`,
   `cgap-parquet footer.rs:84`. Blocked: `ggen` absent, no generated source
   carries the constant, re-export would add a dep edge. Unblock path in E1.
7. **KD-4 stays open** (Ra3). The four sovereign verbs landed; slot
   resolution has not. `oabi_lookup.rs` record now spans 1-50, register
   cites 1-31.
8. **C1 falsifier unwitnessed** (C1). `.rq` gates, ontology, templates
   unrun (`ggen`, `rdflib` absent); SPARQL gates alone admit an empty graph;
   `ddx:order`/`ddx:requires` undeclared; Nav half of gate 030 unmodeled.
9. **Host-bound gates cannot run on Linux** (R4, E2, Ra4). `crates/cli`
   hard-codes `/Users/sac` paths; `ffi_surface_gate.py` is macOS-pathed;
   `zero_string_gate --self-check` exits 1 on objdump format. All UNVERIFIED
   on the target host, not passes.
10. **Next court-hardening candidates** (Ra4): `mlkem768_kem_rt_courts`,
    `cgap-core manifest_court`, `cgap-aoq aoq_v3_golden_bytes_fixture`.
11. **R2 footprint** (R2). 2286 B text baseline covers 4 of 5 exports;
    `wasm_mask_words` cannot build on riscv32imc; real subject is
    wasm32-wasip1, not run.
12. **Cargo target-dir hazard** (Ra4). A sandbox sharing a target dir with
    the shared tree gave a false FAIL; lanes must never share one.

## Not found / not done

- R1: `cgap-ffi` already has 0 rustc warnings; clippy not installed.
- `wire-shm`: delete-first conditions NOT met (E3); keep parked.
- Ra2: penstock residue already retired by L57; replicated at 50.8x
  worst/best on a 4-core host (not comparable across hosts).

## Closed by fix lanes F1-F9 (post-sweep)

| Item | Lane | Result |
|---|---|---|
| Stale CLAUDE.md claims (was #1) | F1 | version, wire-shm/cgap-beam, non-member list, census caveat fixed; wire-shm delete-first clause still names only cgap-mcp (left) |
| KD-7 register + comments (was #2) | F2 | CLOSED-EXPOSED (cgap.h:214 + kd7 court); KD-4 range 1-50; cdylib courts platform-aware |
| closure.rs:230 stack overflow (was #3) | F3 | try_new_boxed; closure_court 3/3, storage suite green; mutation reproduces overflow |
| Vacuous alloc columns ingress/seal (was #5, part) | F4 | AllocProfiler installed; all legs 0-alloc, corroborated |
| C1 pack gaps (was #8) | F5 | .rq gates ALIVE under rdflib (10 mutations); ggen engine still UNVERIFIED |
| Court hardening (was #10) | F6 | 3 more courts name flipping mutations (manifest_court flips dod1 only) |
| zero_string_gate on Linux (was #9, part) | F7+F8 | ELF stub fallback; --self-check exit 0; wave9_a07 passes with artifacts in target/ |
| Flaky egress_ring_court | F9 | test-side race (global alloc counter); per-thread counter; 0/40 fails on re-check |

## Still open after the sweep

- Publish chain (Ra5): cgap-crypto/cgap-audit ^26.11.5 not on registry.
- KD-3 twin (E1) and KD-4 slot resolution (Ra3): ggen absent / A19 not landed.
- cgap_eval_bn3 alloc lever (sovereign.rs:588-622): ABI-visible, owner lane.
- macOS-only paths UNVERIFIED: ffi_surface_gate.py, crates/cli build, Mach-O
  debris leg, cdylib .dylib branch.
- C1: ggen sync x2 replay and Tera rendering; gate 050 only protects wired consumers.
- R2 5-export riscv footprint / wasm32 path; ingress timings contention-suspect.
- wire-shm delete-first clause in CLAUDE.md should name cgap-beam too.
- Remaining un-hardened courts: 37 of the 43 from Ra4's census.
