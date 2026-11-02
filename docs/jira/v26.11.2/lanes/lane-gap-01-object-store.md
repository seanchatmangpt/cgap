# Lane GAP-01 Receipt: object_store wire-in (digest fn injected)

Wave: v26.11.2 starDF · Branch main · Date 2026-11-02 · Lane GAP-01
Standing: **ALIVE** (court + suite run green this session; commands + exits below)

## Files written
- `crates/cgap-git/src/object_store.rs` — digest fn injected (`HashFn`), zero
  external deps preserved; 362→~375 lines.
- `crates/cgap-git/src/lib.rs` — `WIRE-PENDING(lane-git-05)` marker dropped;
  `pub mod object_store;` mounted (lib.rs:146-148 post-edit).
- `crates/cgap-git/tests/object_store_court.rs` — switched from `#[path]`
  mount to the MOUNTED lib path (`use cgap_git::object_store::*`, the
  dispatch's stated "better" option, recorded here); adapted to the injected
  hasher; +1 anti-vacuity court.
- `crates/cgap-git/tests/soc_full_chip_court.rs` — MINIMAL UNBLOCK (disclosed
  both directions): one-line call-site fix at :265
  `ObjectStore::new()` → `ObjectStore::new(cgap_blake3::hash)`. That court
  `#[path]`-mounted the module and was E0061-broken by my API change; fixed
  under the compile-freeze SLA. This file was NOT in my owned list; the
  owning lane supersedes this diff on landing.

## API change (minimal)
- `pub type HashFn = fn(&[u8]) -> Digest;` (tree_synth precedent, follow)
- `digest_of(kind, payload, hasher)`; `ObjectStore::new(hasher)`;
  `ObjectStore::check(kind, payload, claimed, hasher)`; `Default` impl
  removed (no lawful default hasher exists in-crate). Store carries the
  hasher; `insert`/`insert_checked`/`verify`/`ledger_digest` use
  `self.hasher`. No real `[dependencies]` entry added — kernel law holds
  (.claude/rules/kernels-no-std.md).

## Commands + exits (this session, CARGO_TARGET_DIR=/tmp/gap01-target)
- `cargo build -p cgap-git` → exit 0
- `cargo test -p cgap-git --test object_store_court` → **7 passed / 0
  failed, exit 0** (6 original courts, semantics unchanged, +1 new).
- Full per-target census (29 targets + lib, all exit 0): a2a_swarm 4,
  admission_gate 6, commit_synth 10, delta_refusal 4, delta_resolver 8,
  egress_ring 5, envelope_decimal 2, fsck 2, fuzz_codec 4, git_interop 3,
  ingress_ring 6, ladder_mutations 11, large_offset 6, lineage_workflow 1,
  mcp_driver 6, **object_store_court 7**, pack_demux 10, packfile_fuzz 2,
  patent_court_git 4, pkt_line 10, posh_alloc_sentry 1, posh_refusal_matrix 4,
  posh_wire 3, ref_store 5, retract 8, soc_full_chip 1, tree_synth 6,
  lib tests 61. Ignored-by-design: ladder_timing_stress (1), perf_baseline (1).
- Aggregate `cargo test -p cgap-git` aborts at `e2e_wire_substrate_court`
  (E0382 `receipt` moved, tests/e2e_wire_substrate_court.rs:230) — compile
  abort, NOT a failure count (census law). NOT this lane's break: that court
  does not touch object_store; line 17-21 of that file names GAP-01 as
  future wire-in but its E0382 is a LeaseReceipt move error in another
  lane's in-flight surface. LOUD, unresolved, owner is whoever landed
  LeaseReceipt::seal. Aggregate census therefore ran per-target (--no-fail-fast
  does not survive compile aborts).

## Anti-vacuity (task 4) — flip witnessed in /tmp sandbox
Mutation: `/tmp/gap01-sandbox/cgap-git` (sandbox copy), removed the
duplicate-rejection block in `insert_prehashed`. Result: 2 FAILED
(`stale_constant_hasher_is_rejected_not_laundered`,
`capacity_and_duplicate_courts`), 5 passed. Restore not needed — the
shared tree was never touched; sandbox discarded at cleanup.

New court `stale_constant_hasher_is_rejected_not_laundered`: a hasher that
ignores its input (constant `[0x5A; 32]`) is detectable — second distinct
insert is typed `Duplicate`, `insert_checked` with the honest real-BLAKE3
digest is typed `DigestMismatch`, and a control store under the real hasher
separates the same payloads. Control-in-test witness pair, plus the sandbox
mutation above proves the court itself can flip.

## Court semantics preserved (6/6)
The six GIT-05 courts run unchanged through the lib path with the real
`cgap_blake3::hash` bound as the injected `HashFn` (`const BLAKE3: HashFn =
cgap_blake3::hash`). Zero-heap source-scan court still green (no banned
tokens introduced).

## Fences
- Must-not-regress fences: all 29 cgap-git test targets + lib (run, listed
  above, exit 0 each); cargo build -p cgap-git exit 0; zero external deps
  (Cargo.toml untouched — [dependencies] stays empty, witnessed).
- Cross-disclosure (disclose-in-both-directions law): soc_full_chip_court
  one-line unblock disclosed here and cross-referenced to
  lanes/lane-fix06-git-wirein.md (the WIRE-PENDING marker this lane drops)
  and to the soc court's owning lane (GIT-16). object_store_court.rs edit is
  an in-lane file adaptation named in this receipt; fix06's receipt said
  object_store's court "compiles the module independently and is green" via
  #[path] — that statement is superseded by this receipt (court now uses the
  lib path).
- UNVERIFIED: none withheld. Not claimed: Git parity (unchanged, see
  CANONICAL_HEADER_LAW), perf numbers (no bench in scope).

## Cleanup law
Build-root leases deleted at lane end (witnessed below in commands):
`/tmp/gap01-target`, `/tmp/gap01-sandbox-target`, `/tmp/gap01-sandbox`,
`/tmp/gap01-targets.txt`, `/tmp/gap01-suite.log`.
