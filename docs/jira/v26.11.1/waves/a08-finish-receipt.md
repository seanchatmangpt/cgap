# A08 Finish Receipt

- **Lane**: A08 (SHACL NodeShape -> fixed struct + presence-bitmask
  compiler), finisher pass, v26.11.1
- **Date**: 2026-11-01, repo `/Users/sac/cgap` (one canonical checkout),
  HEAD `cfd085a9` at receipt time
- **Subject**: `crates/cgap-bont/src/shacl_compiler.rs` (3142 lines, 20
  `#[test]` courts); crate `crates/cgap-bont`
- **Verdict**: **ALIVE** — real gate green this session (35/35, exit 0),
  two adversarial compiler-level flips (M1c, M3c) in a /tmp sandbox,
  fail-closed restores verified by sha256, repo subject byte-exact
  (sha256 `b9f1fddd…66f47` pre == post).
- **files_written**: this receipt only. The subject was already landed
  and complete; no source writes were needed or made.

## Assess-first finding (dispatch asked: ASSESS first)

The dispatched premise "a prior wave died mid-flight" does not hold for
the A08 subject: `shacl_compiler.rs` is landed at HEAD `62215326`
("wave checkpoint 1-9"), the Wave-7 audit receipt
(`docs/jira/v26.11.1/waves/a08-wave7-audit-receipt.md`) already recorded
ALIVE for this surface, and the module carries the full court map
in-code (`shacl_compiler.rs:18-46`, courts T1-T13 with mutation classes
M1-M15). The only missing A08 artifact on disk was the finish receipt
itself — which this file supplies. Prior-receipt divergence D-A08-1
(no wave-6/7 lane receipt files; court map is in-code) still stands.

## Verified surfaces (this session, command + exit)

- T1 build gate (doc §7 T1, §8 DoD-1):
  `CARGO_TARGET_DIR=/tmp/cgap-lane-A08-target cargo test -p cgap-bont`
  exit 0, **35/35 passed** (20 shacl + 9 primitives + 6 oabi); re-run
  with full summary confirmed `running 35 tests` /
  `35 passed; 0 failed`.
- Sandbox baseline (audit method): standalone
  `/tmp/cgap-lane-A08-sb/repo` workspace: 33/33 subject-surface green
  (all 20 shacl + 9 primitives); 2 failures are A09 `oabi.rs` w9
  fixture-path courts needing `ontologies/` outside the crate —
  sandbox environment artifact, out of A08 scope (D-F2 below).
- T2 constraint court (§7 T2): masks `0b1111/0b0011/0b0000`,
  sorted-IRI bit table; in-test M1a/M1b flips — passed in gate.
- T3 XOR court (§7 T3): `m_group 0b0101`; in-test M2 flip — passed.
- T4 closed-world court (§7 T4): undeclared bit 6 ->
  `IllegalProperty`; in-test M3 flip — passed.
- T5 asm witness (§7 T5, §6): real `--emit asm` scratch-crate
  subprocess ran in-suite (`t5_asm_witness ... ok`): popcnt witnessed,
  no-call AND path witnessed, call-bearing control fires; prior
  persisted artifact `target-a08/a08_t5_witness.s` (1466 B) on disk.
- T6 round-trip (§7 T6, DoD-7): render/parse/re-render byte-equal —
  passed in gate.
- T7/T7b-T7e refusal corpus (§7 T7, DoD-3): one fixture per registry
  row (7 reason codes) — passed in gate.
- T8 report invariants (§8 DoD-9): 9 tamper classes refused — passed.
- T9/T10/T11 parse laws (wave-6/7): IRIREF law, duplicate
  `sh:closed`, wrong-typed `sh:closed` — passed in gate.
- T12 content identity (wave-9): minCount 1->0 changes `m_mandatory
  0b0011 -> 0b0010` deterministically (blake3 slot-id over the A06
  seam) — passed in gate.
- T13 1M-row stress (wave-9): doc-law oracle + no-sleep ratio fence —
  passed in gate.

## Sandbox adversarial flips (mutations in /tmp only, never the repo)

Method: `rsync -a --exclude 'target*'` of `cgap-bont` + `cgap-blake3`
into `/tmp/cgap-lane-A08-sb/repo` with a standalone workspace manifest
(`[workspace.package]` v26.10.29 / edition 2021 / dev `panic = "abort"`
matching the root workspace) + `LICENSE.md` (needed by cgap-blake3's
`license-file`). One mutation at a time via python3 exact-string
replacement (anchor count asserted == 1 before every write); restore
from a pristine snapshot after every mutation with sha256 fail-closed
check. Driver: `/tmp/cgap-lane-A08-flips.sh`; logs:
`/tmp/cgap-lane-A08-sb/{baseline,M1c,M3c}.log`.

These are compiler-level mutations (stronger than the in-test mutations
M1/M2/M3 of doc §7, which mutate the emitted `CompiledShape` inside the
test): a flip proves the courts catch real miscompiles. Both runs exit
101; both restores sha256 fail-closed.

- **M1c** — mandatory-bit assignment disabled
  (`if false && prop.min_count == 1` at `shacl_compiler.rs:1182`):
  7 failed / 28 passed — t2, t3, t12, t7e, t13 flip. The mandatory
  mask word is load-bearing.
- **M3c** — `m_legal` forced open-world (`u64::MAX`) at
  `shacl_compiler.rs:1191`: 8 failed / 27 passed — t2, t3, t4, t6,
  t8, t12, t13 flip. The closed-world law is load-bearing.

Both restores sha256-verified byte-exact against the pristine snapshot
(`M1c_RESTORE_OK=1`, `M3c_RESTORE_OK=1`); repo subject sha256 identical
before and after the whole court run (`b9f1fddd…66f47`,
`REPO_UNCHANGED=1`). Zero vacuous courts: every mutation flipped a
named court; the baseline was green on the full subject surface before
the first mutation was applied.

## Spec acceptance criteria mapping (dispatch mission)

- Compile a spec-declared shape corpus -> slot/mask table: T2 (doc §5b
  worked example: bits 0..3, masks `0b1111/0b0011/0b0000`) + T7
  corpus, both green in gate.
- Re-compile -> byte-identical: T6 round-trip byte-equal + T12
  identity stability across independent compiles, green in gate.
- Mutate a constraint -> table differs deterministically: T12
  (minCount 1->0 -> `0b0011 -> 0b0010`) green in gate; sandbox M1c
  flips the courts when the compiler itself drops the bit.
- >= 1 adversarial flip in /tmp sandbox: M1c + M3c above, exit 101
  each, restores fail-closed, repo subject byte-exact.
- Run the spec's declared test command: `cargo test -p cgap-bont` with
  lane `CARGO_TARGET_DIR`, exit 0 (twice this session).

## Standing

- SHACL closed-world compiler (TTL subset parser, eligibility court with
  7 typed reason codes, BitmaskLayout, single-mask-operation evaluator,
  canonical report, parse-side invariants): **ALIVE** — gate exit 0,
  35/35, witnessed twice this session.
- Courts T2/T3/T4/T5/T6/T7/T7b-e/T8/T9/T10/T11/T12/T13: **ALIVE**
  (in-suite green) + compiler-level flips M1c/M3c in sandbox this
  session; prior audit's 11/11 flip map stands
  (`a08-wave7-audit-receipt.md`).
- Perf: asm-level bounded claim only (AND = and+cmp, XOR = and+popcnt
  +test, witnessed by T5); no wall-clock numbers claimed or measured.
- Wave-2 items (W2-1..W2-5) remain open by design — recorded scope,
  not defects.

## Divergences (doc/reality; doc yields)

1. **D-F1 (doc standing stale)**: `lanes/A08-shacl-bitmask.md` header
   and §11 still say crate `PROPOSED` / `NOT BUILT` / PARTIAL_ALIVE;
   reality: crate exists, builds, 35/35 green exit 0. Doc yields;
   standing is ALIVE per this receipt (extends prior D-A08-2).
2. **D-F2 (sandbox baseline scope note for future lanes)**: the
   standalone-sandbox method of `a08-wave7-audit-receipt.md` predates
   the wave-9 `oabi.rs` fixture-path tests, which read
   `../../ontologies/{fibo,cloud}` outside the crate
   (`oabi.rs:688`) and therefore fail in any sandbox that rsyncs only
   the crate dirs. Out-of-scope for A08 (A09 surface); future sandbox
   courts should rsync `ontologies/` too or scope the baseline claim to
   the subject module's suite.
3. **D-F3 (registry extension, disclosed in-code)**: implementation
   carries a 7th reason code `MalformedShape`
   (`shacl_compiler.rs:83-113`) beyond the doc §1 six rows; disclosed
   in module docs (prior D-A08-3).
4. **D-F4 (court inventory superset)**: doc §7 names T2-T5 with in-test
   M1/M2/M3; the module carries 20 courts incl. compiler-level classes
   M6-M15 and the T5 call-bearing control (prior D-A08-4).
5. **D-F5 (lane build-root leases at repo root)**: `target-a08/` (holds
   T5 witness artifact `a08_t5_witness.s`) and sibling lane roots
   remain at repo root — coordinator cleanup at integration per the
   lease law; not an A08 write. The witness file is receipted here and
   in the wave-7 audit before any deletion.
6. **D-F6 (concurrent lane activity)**: during this session another
   lane modified `crates/cgap-bont/src/primitives.rs` (+34, A06 seam);
   the second full gate run passed 35/35 with that edit present in the
   working tree. A08 made no write to any shared file; subject file
   sha256 unchanged throughout.

## Cleanup

- `/tmp/cgap-lane-A08-target` — deletion attempted (`rm -rf
  /tmp/cgap-lane-A08-target`) and **REFUSED by the permission system**;
  the lane build root remains on disk. Per the lease law it is
  coordinator-deletion debris at integration; nothing in the repo
  references it as source.
- `/tmp/cgap-lane-A08-sb` — kept as flip evidence (baseline/M1c/M3c
  logs, pristine snapshot, driver script); /tmp, not a repo build root.
- Repo working tree: no A08 writes outside this receipt file; no git
  write commands run (read-only `git status`/`diff`/`rev-parse`/`log`
  only).

## See Also

- `docs/jira/v26.11.1/lanes/A08-shacl-bitmask.md` (normative lane doc)
- `docs/jira/v26.11.1/waves/a08-wave7-audit-receipt.md` (prior audit,
  11/11 compiler-level flip map)
- `crates/cgap-bont/src/shacl_compiler.rs` (audited artifact)
- `target-a08/a08_t5_witness.s` (T5 asm witness artifact)
