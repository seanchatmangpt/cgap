# p2-asm — 0 blr / 0 br xN / 0 brk census, seven hot regions

Lane p2-asm. Date 2026-11-01. Read-only lane (no git writes, no publishes).
Method per `docs/jira/v26.11.2/waves/w3-t2-asm-census.md`:
`CARGO_TARGET_DIR=/tmp/p2-asm-<crate> cargo rustc --release -p <crate> -- --emit asm`
+ python symbol-bounded regex census (`^__?[A-Za-z_$][\w$.]*:$` region labels);
objdump -d used only on the real p28 release test binary (rlib objects are
LLVM-bitcode wrappers, objdump refuses them). Toolchain nightly-aarch64-apple-darwin.

## Ladder

| Gate | Command (cwd /Users/sac/cgap) | Exit | Evidence |
|---|---|---|---|
| build (5 crates) | CARGO_TARGET_DIR=/tmp/p2-asm-target cargo build --release
-p cgap-bn3 -p cgap-arrow -p cgap-storage -p cgap-gate -p cgap-p28 | 1 (bont
excluded) | bont was mid-edit (unclosed delimiter, shacl_mask.rs:339); rest ok |
| build (bont, poll) | CARGO_TARGET_DIR=/tmp/p2-asm-target cargo build --release
-p cgap-bont | 0 | Finished release in 1.48s (owner fix landed ~2 min later;
poll deadline 60 min) |
| emit asm (6 crates) | per-crate `cargo rustc --release -p <crate> --
--emit asm` | 0 x6 | bn3 325 KB, arrow 469 KB, storage 113 KB, gate 4 KB,
p28 34 KB, bont 678 KB |
| p28 test build | CARGO_TARGET_DIR=/tmp/p2-asm-p28 cargo test --release
-p cgap-p28 --no-run | 0 | binary at release/build/cgap-p28/13240a9e14eebf28/out/ |
| objdump + census | objdump -d <p28 test bin> + python region census | 0 |
table below |

## Counts (blr / br xN / brk per named region)

- bn3 radix semi-join (unify.rs): `semi_join_masks` (unify.rs:750) is fully
  inlined — zero asm symbols; census taken over its only callers:
  - `unify::emit_join_masks` (bl=15, 446 lines): blr=0 br_xn=0 brk=0
  - `unify::bucket_candidates` (bl=6, 133 lines): blr=0 br_xn=0 brk=0
  - `unify::unify_batch_probe` (bl=115, 2528 lines): blr=0 br_xn=0 brk=0
- bn3 closure (unify.rs:1319/1328):
  - `unify::unify_to_fixpoint` (5 lines, thunk): blr=0 br_xn=0 brk=0 bl=0
  - `unify::unify_to_fixpoint_opt` (1214 lines): **blr=1** — CRITICAL,
    coordinates: bn3 asm `__RNvNtCsdoP6KDw1Z0r_8cgap_bn35unify21unify_to_fixpoint_opt`,
    .s line 10230, block LBB68_43: `ldr x8, [x0]; blr x8` immediately after the
    `__RNvNCNKNvNvMNtNtCsfhISatJ9aHf_3std4hash6randomNtBa_11RandomState3new4KEYS0s_023___RUST_STD_INTERNAL_VAL@TLVPPAGE`
    TLS-gated load (exact mangled symbol, load-bearing; 104 chars).
    Provenance: std `RandomState::new` once-init inlined from std
    (thread-local std-runtime gate), not a cgap dispatch site — but it IS in-region,
    so it is reported CRITICAL per the lane contract, not waived.
- bont SHACL masks: `MaskPlan::evaluate` (shacl_mask.rs:128) fully inlined into
  `shacl_mask::brute_force` (489-line region): blr=0 br_xn=0 brk=0, bl=18 direct.
  Out-of-scope note: the `sort_unstable_by_key` monomorphizations instantiated BY
  brute_force (ipnsort/quicksort glue in the same .s) contain 2 `brk` unreachable
  traps — std sort internals, outside the named region, listed for honesty.
- storage anti-join: **MISSING** — no anti-join kernel exists in cgap-storage
  (`grep -rli anti_join crates/cgap-storage/src` = 0 hits; storage asm symbol table
  has zero anti/anti-join symbols; 0 `anti` byte matches in the .s). This cell is
  loudly MISSING, not a skip. Adjacent censused cells (same crate):
  `closure::semi_naive_closure`, `closure::naive_closure`,
  `closure::mutant_base_only_count` — blr=0 br_xn=0 brk=0 each.
- arrow IPC emitter (ipc.rs): `ipc::emit` (bl=7, 175 lines), `ipc::read`
  (bl=2, 551 lines), `ipc::seal_meta` (bl=0, 171 lines): blr=0 br_xn=0 brk=0 each.
- gate prefilter8+ingress (penstock.rs): `prefilter8` (penstock.rs:677) fully
  inlined into `penstock::ingress` (204-line region): blr=0 br_xn=0 brk=0 bl=0.
- p28 fused_row (asc450.rs:967): generic over `S: ProofSealer`, so no lib-rlib
  symbol; censused via objdump of the release test binary — monomorphization
  `asc4509fused_rowNtB2_16LeaseEpochSealerEB4_` (213 lines): blr=0 br_xn=0 brk=0,
  bl=4 direct. L5 court caller `w3l5_fused_row_byte_equality_and_anchor_court`
  (734 lines): blr=0 br_xn=0 brk=0.
- p28 lib surface re-confirmed (all 10 asc450 symbols incl. epoch_binding,
  seal/verify pair, from/to_bytes): blr=0 br_xn=0 brk=0 — matches w3-t2 receipt.
- gate ingress note: the gate .s is 4 KB with ingress as the only function symbol —
  the crate's penstock surface is effectively one inlined entry; census covers it.

## Verdict

PARTIAL — six of seven contracted regions are CLEAN (0 blr / 0 br xN / 0 brk):
bn3 semi-join callers, bont SHACL masks, arrow IPC emitter, gate prefilter8+
ingress, p28 fused_row, storage closure cells. Two findings block ALIVE:
1. CRITICAL: 1 in-region `blr x8` in bn3 `unify_to_fixpoint_opt` (std
   RandomState::new TLS once-init inlined from std; coordinates above).
   Fix direction: the fixpoint loop must not mint a `RandomState` per call —
   hoist/pass a seeded hasher so no std TLS init is reachable from the region.
2. MISSING: storage anti-join kernel does not exist; region cannot be censused.

Perf claims in this receipt: none — census only, no timing ran.

## Notes

- bont was mid-edit at lane start (unclosed delimiter shacl_mask.rs:339,
  `let s2 = sid(u64 -> vec placeholder)`); compile-freeze SLA honored: owner
  repaired within ~2 minutes, build then exit 0. Census ran on the repaired tree.
- `-o` on `cargo rustc -- --emit asm` appends the extra-filename suffix to the
  output name (observed: requested `/tmp/p2-asm-bont.s`, got
  `/tmp/p2-asm-bont-a8d630f23e8e502b.s`); flag-free emit into the target dir is
  the reliable route (w3-t2's method, confirmed).
- Out-of-region brk honesty list (not counted, out of scope): bn3 ipnsort
  unreachable traps (same class as w3-t2 noted), bont sort glue brk x2,
  storage sort glue.
- Build roots /tmp/p2-asm-{target,bn3,arrow,storage,gate,p28,bont,bont2} and
  asm scratch deleted via shutil.rmtree / os.remove at lane end; other lanes'
  /tmp/p2-* entries left untouched.

## JSON footer

{"lane":"p2-asm","files_written":["docs/jira/v26.11.2/waves/p2-asm-0blr-census.md"],"commands":[{"cmd":"cargo build --release -p cgap-bn3 -p cgap-arrow -p cgap-storage -p cgap-gate -p cgap-p28","exit":1,"note":"bont excluded; 5 crates ok"},{"cmd":"cargo build --release -p cgap-bont","exit":0},{"cmd":"cargo rustc --release -- --emit asm x6 crates","exit":0},{"cmd":"cargo test --release -p cgap-p28 --no-run","exit":0},{"cmd":"objdump -d p28 test binary + python census","exit":0}],"courts_run":[],"standing":"PARTIAL","divergences":[{"region":"bn3 unify_to_fixpoint_opt","finding":"CRITICAL: 1 in-region blr x8 at bn3 .s line 10230 (LBB68_43), std RandomState::new TLS once-init inlined from std"},{"region":"cgap-storage anti-join","finding":"MISSING: no anti-join kernel exists; cell loudly missing"},{"region":"clean","finding":"bn3 semi-join callers, bont shacl_mask::brute_force, arrow ipc::emit/read/seal_meta, gate ingress(+inlined prefilter8), p28 fused_row monomorphization, storage closure cells: 0 blr / 0 br xN / 0 brk"}]}
