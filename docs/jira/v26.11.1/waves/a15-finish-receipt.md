# A15 Finish Receipt

- **Lane**: A15 (POWL v2 transition verifier + OCEL 2.0 columnar flight
  recorder), v26.11.1 — finisher/assessment pass over the landed artifact
- **Subject**: repo `/Users/sac/cgap`, branch `main`, HEAD `cfd085a9`,
  working tree carrying only concurrent sibling-lane edits outside this
  lane's lease; lane lease `crates/audit/src/` CLEAN (all three files
  byte-identical to HEAD)
- **Mode**: failed-lane restart assessed, then finished: no lane source
  changes were needed — commit `9be9109d` landed the Wave-1 ABI hygiene
  (`BindingIndex.entries` `pub(crate)` -> private, `powl.rs` only) and
  the wave-9 courts were already in-tree; this session ran the real
  gate, the D3 reorder flip in a /tmp sandbox, and pins the finish
  receipt
- **Verdict**: **ALIVE** — real gate
  `CARGO_TARGET_DIR=/tmp/cgap-lane-A15-target cargo test -p cgap-audit`
  exit 0, **37/37** passed (5 alloc_counter + 15 powl + 17 ocel2_slab,
  including both 1,000,000-row stress courts), run fresh this session;
  sandbox flip M-D3-R isolated to exactly the D3 court (36 pass / 1
  fail, exit 101), restore byte-exact (sha256 fail-closed), re-run
  37/37 green

## Gate (command + exit, this session)

- `CARGO_TARGET_DIR=/tmp/cgap-lane-A15-target cargo test -p cgap-audit`
  -> exit 0, 37 passed / 0 failed / 0 ignored.
- Same gate with `-- --nocapture --test-threads=1` filtered to
  `a20_stress verdict_is_deterministic check_path_is_allocation_free
  string_leak_court` -> exit 0, 5 passed, stress timings printed.

Measured observations (dev profile, single run, this host — measured,
never targeted): `a20-1m-stress(ocel2): rows=1000000 objects=1000
append=0.275s seal=0.623s open=0.792s slab_bytes=4054932`;
`a20-1m-stress(powl): rows=1000000 bindings=1000 verify=0.635s`.

## Mission items -> witnessed surfaces

1. **Partial-order workflow traces, zero runtime drift.** Witnessed by
   `a04_identity_court_seal_open_reseal_bit_exact` (two independent
   seals of one log byte-identical; open->re-seal bit-for-bit; header
   checksum equals BLAKE3 over the on-disk payload; different-log
   control flips the identity) and `ocel_append_court_seal_reopen_column_equal`
   (real append path -> seal -> re-open, column-equal over every
   column). A spec-declared trace recorded through the real append path
   replays byte-identical.
2. **ocel2_slab conformance per spec §4/§6.** The full §6.1 court set
   passed this session: compression round-trip + `compress-drop`
   (9-row mutant refused by the row pin, `RowPinMismatch {pinned: 9,
   got: 10}`); `ocel-mutate` (payload byte / header count / magic
   classes caught, byte-exact restores); `string-leak` (append
   zero-alloc on a reserved log, unreserved control allocates); intern
   D1/D2/D4/D5 courts; forged-header (`CapacityExceeded` BEFORE
   allocation); intern-version; trailing-bytes; A14 dam precedence;
   A05 power-cut sweep (every prefix + every single-byte flip, exact
   tear-class counts: 64 header cuts, n-64 payload cuts, 55 refused
   header bytes, exactly 9 documented v1 tolerances); A20 1M stress;
   export court through the REAL `audit_ocel2_append` symbol.
3. **CountingAlloc zero-alloc path + D3 determinism law.**
   `check_path_is_allocation_free_beyond_the_binding_index` (check path
   zero-alloc; anti-vacuity control: `index_bindings` on the same log
   DOES allocate — the court measures); `string_leak_court_append_is_allocation_free`;
   `verdict_is_deterministic_across_repeated_index_builds` (64 fresh
   index builds -> identical verdict, evidence pinned to event id 1).

## Sandbox flip (mission item 5) — all mutations in `/tmp/a15-finish-sb`

Method: standalone workspace at `/tmp/a15-finish-sb` (8-crate dep
closure `audit, cgap-arrow, cgap-parquet, cgap-core, cgap-gate,
cgap-storage, cgap-blake3, cgap-git` + workspace stub v26.10.29 /
edition 2021 / dev `panic = "abort"`). Sandbox sources sha256-identical
to repo at staging (`powl.rs 3b95aab4fa7126c2`, `ocel2_slab.rs
9774419bd3ba0930`, `alloc_counter.rs 9df82a2bebfa4007`). Baseline 37/37
green exit 0. One mutation via python3 exact-string replacement (anchor
count asserted == 1 before the write); restore from `.pristine` with
sha256 fail-closed compare; sandbox == repo byte-exact after restore.

- **M-D3-R (reorder)**: `entries.sort_by_key(|b| b.events[0]);` ->
  `entries.sort_by_key(|b| core::cmp::Reverse(b.events[0]));` —
  binding entries reordered (descending first-event-row).
- **Flip**: `verdict_is_deterministic_across_repeated_index_builds`
  FAILED — `left: evidence [3, ...]` != `right: [1, ...]`; the
  descending order surfaces binding 200's violation instead of binding
  100's, so the pinned evidence id flips. Exit 101.
- **Isolation**: full mutated suite 36 passed / 1 failed, exit 101 —
  the reorder flips EXACTLY the D3 court and nothing else.
- **Restore**: sha256 fail-closed, all three lease files byte-exact to
  repo; suite re-run 37/37 green exit 0.

The flip law holds both ways: removal of the sort reintroduces
HashMap-order nondeterminism (wave-8 M-D3, flip witnessed then);
deterministic-but-descending order flips the pinned evidence id
(witnessed this session). A court that cannot see a reordered binding
walk is decoration.

## Divergences (doc-vs-reality; spec yields, recorded)

1. **§6.3 gate feature flag is stale**: the spec gate is
   `cargo test -p cgap-audit --features ocel2`, but
   `crates/audit/Cargo.toml` has no `[features]` section and `lib.rs`
   declares `powl` / `ocel2_slab` unconditionally — the gate was run
   without the flag (the flag would error "no feature named ocel2").
   The spec's feature-gated framing predates the landed unconditional
   module.
2. **"fixed-width columnar rings" (dispatch) vs slab (spec)**: the spec
   §2.2/§4 defines a fixed-width columnar SLAB, not a ring; no ring
   structure exists in the lane surface. The drift court is the A04
   identity court. Spec is authoritative per the lane contract; the
   dispatch term is recorded, not implemented.
3. **Sandbox dep closure grew since wave 8**: the wave-8 receipt built
   its sandbox from 3 crates; `cgap-arrow` gained `cgap-core` /
   `cgap-gate` / `cgap-storage` deps in the 9be9109d wave 9-10 landing,
   so this session's closure needed 8 crates. Two failed baseline
   attempts before the closure was complete (missing workspace
   `authors`; missing `cgap-core` manifest) — recorded, both fixed
   sandbox-only, zero repo impact.
4. **`alloc_counter.rs` lease provision unused**: the spec (§1.1) and
   dispatch name it as the CountingAlloc harness home; it already
   existed complete with its own court set and needed no change.
5. **No lane source changed this session**: `files_written` is the
   receipt only. The lane artifact landed across prior waves (last
   touched by `9be9109d`); this pass is assessment + gate + flip +
   receipt — ALIVE is claimed on the witnessed run, not on the
   prior-wave claim.
6. **`crate-type` rlib note (pre-existing, disclosed by the crate)**:
   the `rlib` entry was added by lane 18 as a compile-freeze minimal
   unblock (disclosed in both receipts per the fan-out law);
   `cdylib` build unchanged, and the export court still exercises the
   real `#[no_mangle]` symbol this session.

## Cleanup (lease law)

`/tmp/cgap-lane-A15-target` deleted after the final gate run;
`/tmp/a15-finish-sb` (sandbox workspace + its target dir) deleted —
lane build roots are leases, not assets. Repo left with ONLY
`docs/jira/v26.11.1/waves/a15-finish-receipt.md` added.

## See Also

`docs/jira/v26.11.1/lanes/A15-powl-ocel2.md` (authoritative spec) ·
`docs/jira/v26.11.1/waves/a15-wave8-audit-receipt.md` (wave-8 defect
repairs D1-D6) · `~/.claude/rules/testing-chicago-style.md` (court
discipline) · `~/.claude/rules/no-overclaiming-rust.md` (status
vocabulary)
