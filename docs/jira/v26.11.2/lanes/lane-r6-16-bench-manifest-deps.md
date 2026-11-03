# Lane R6-16 — bench manifest unused-dep cleanup

Standing: ALIVE (all commands + exits this session, /tmp/r6-16-target lease, deleted at end).

## W1 Identity
- Subject: `crates/bench/Cargo.toml` (sole owned file).
- Census claim (R-01/FS-17): 3 unused-dependency manifest warnings
  (`cgap-ffi`, `cgap-git`, `cgap-storage`).

## W2 Grounding
- Grep census (this session): `cgap_ffi` used in `benches/{mctq_consolidation,seal,chains-sovereign}.rs`;
  `cgap_git` in `benches/challenger_git.rs`; `cgap_storage` in
  `benches/{dred_churn,closure,mat_closure}.rs`. Zero `src/bin` usage of
  ffi/git/storage/core/gate/bn3/p28/crypto. arrow/blake3/parquet are
  used in `src/main.rs` + `src/bin` (non-dev targets) — stayed in
  [dependencies].
- Dispatch told "remove only if grep confirms no references" — grep
  REFUTED removal: all three are live bench deps. Divergence recorded.

## W3 Consequence (diff)
- No dependency deleted. All 8 bench-only deps (ffi/git/storage/core/gate/
  bn3/p28/crypto) moved [dependencies] -> [dev-dependencies], with an
  on-disk comment explaining cargo's `unused_dependencies` lint semantics.
- Diff summary: [dependencies] = arrow, blake3, parquet(std); [dev-deps]
  = core, gate, bn3, p28, crypto, ffi, git, storage, divan =0.1.21.

## W4 Commands / exits
1. `cargo build -p bench` (pre-edit): exit 0, 3 dep warnings (ffi/git/storage).
2. Edit 1 (move the 3): 3 gone, but 4 NEW warnings (core/gate/bn3/p28) —
   cargo's lint counts transitive appearance as use; cgap-ffi in
   [dependencies] shadowed them. Manifest warnings are conservation-law:
   moving a shadowing dep unmasks its shadowees.
3. Edit 2+3 (move core/gate/bn3/p28/crypto likewise): 0 dep warnings.
4. `cargo build -p bench`: exit 0, ZERO warnings.
5. `cargo bench -p bench --no-run`: exit 0 — all 21 [[bench]] targets
   compiled (incl. chains-*, mat_closure, dred_churn, closure, bgp,
   mctq_consolidation).
6. Lease cleanup: rmtree /tmp/r6-16-target, exit 0.

## W5 Warning delta
3 -> 0 manifest dependency warnings (7 deps moved dev-ward, 0 deleted).
Non-dep warning (`workspace.package.rust-version` lint) is workspace-root
scope, out of lane scope, unchanged.

## W6 Fences / anti-vacuity
- Fence: every [[bench]] target must still compile — run (W4.5, exit 0).
  Mutation that would flip it: deleting a moved dep makes the referencing
  bench fail to compile (grep evidence at W2 names the files).
- Fence: root workspace must stay green — bench manifest change is
  crate-local; dev-deps cannot affect other members' builds.
- Concurrent-lane disclosure: the Edit tool reported on-disk changes to
  the shared manifest mid-lane (another lane's edit landed between my
  Read and first Edit — rust-version.workspace line appeared at :6 and
  crypto shifted). My edits applied cleanly on the newer base; no
  conflict, no SLA unblock needed, neither lane suppressed the other.

## W7 Replay
- Re-run: `cargo build -p bench 2>&1 | grep -c 'consider removing the dependency'`
  -> 0; `cargo bench -p bench --no-run` -> exit 0.
