# BRANCHLESS-REUSE-MASTER-MAP

Lane BR12 synthesis. Inputs: all 11 lane maps
`docs/jira/v26.11.2/explore/branchless-reuse-lane-01..11.md` (read fully this
session), plus receipts `waves/w6-b6-census-court.md`,
`adversarial/forensic-lane-06.md` (FA6), FA2/FA3/FA10 verdicts as cited by the
lanes themselves. Read-only synthesis: no source edits, no builds, no
Process.sleep. Every row cites its lane map + row/section id.

Standing of this map: UNVERIFIED as a whole — every gain below is
[ANALYTIC]/TARGET until an implementing lane measures it. The lanes'
honest verdicts (REJECT / NO-SUBSTITUTION / SKIP) are preserved verbatim in
intent; a REJECT here is not a backlog item.

## 1. Master Substitution Matrix

Columns: Subsystem | Exact File:Line | Current Logic | Proposed Primitive |
Expected Gain class | Anti-Vacuity Court.

| Subsystem | File:Line | Current Logic | Proposed Primitive | Gain | Court |
|---|---|---|---|---|---|
| gate S1 | penstock.rs:536,541 L01 | 2 exit cmps | swar_eq_u64 fold | ANALYTIC | MutA corrupt 101 |
| gate S2/S5 | penstock.rs:550,651 L01 | zero scans | REP5 vl-zero | ANALYTIC | MutB invert 101 |
| gate S4 | penstock.rs:573 L01 | slice eq | swar_eq_u64 fused | ANALYTIC | MutC SPOLIATION |
| gate S6 | penstock.rs:614-5 L01 | prefilter8 | REP5 prefilter8 | receipt | MutD mix const 101 |
| gate S7 | penstock.rs:615 L01 | mask+test | KEEP poison_mask | 0 | MutD covers |
| gate stay | penstock.rs:524..598 L01 | typed refusals | MUST-STAY branchy | 0 | refused |
| gate S8 | FFI caller L01 | quota site | poison_mask FFI | ool | [SKIP] FFI court |
| gate S9 | penstock.rs:620 L01 | byteloop loads | REP5 keeps form | ANALYTIC | MutE slice regress |
| p28 M2 | lease.rs:115-126 L02 | tick guard | Tick(u64) stronger | 0 | M2 invert 117 panic |
| p28 cold | asc450.rs:824+ L02 | checked chains | NO-SUBSTITUTION | 0 | 1613 kills sat mut |
| arrow B2 | bitmask.rs:126-134 L03 | perbyte popcnt | const popcnt | ANALYTIC | count_zeros flip |
| arrow B2b | bitmap.rs:103-117 L03 | per-bit set | SWAR byte pack | ANALYTIC | i>>2 mut; rt fails |
| arrow rej | bitmap.rs:75-90 L03 | checked loads | REJECT | 0 | select_slice REJECT |
| pq T1 | thrift.rs:81-97 L04 | byte varint | SWAR varint | m-cold | sandbox typed flip |
| pq T2 | thrift.rs:199-223 L04 | delta fld hdr | none | m-cold | nibble MissingReq |
| pq T3 | thrift.rs:475+ L04 | dense match | const tbl | net-neg | refused |
| pq C1 | scan.rs:331-335 L05 | per-RG OR | mask bulk-skip | marginal | invert; court flips |
| pq C2 | scan.rs:832 L05 | hot value pred | mask push | val-lane | FA8 covers 668 |
| pq R1 | footer.rs:119-163 L06 | bounds refusals | REJECT fail-open | 0 | clamp FAIL/VACUOUS |
| pq P1 | mmap_writer.rs:359-72 L06 | chk/byte | 8-byte group emit | TGT 13ms | no fallback flip |
| pq P2 | mmap_writer.rs:335-57 L06 | chk/varint | varint_len hoist | TGT | skip chk flip |
| pq R2r | mmap_writer.rs:436-48 L06 | run scans | REJECT | 0 | byte-exact court |
| bn3 radix | unify.rs:825-828 L07 | private dup | radix_bucket_12 pub | dedup | B5 y>>8 flip |
| bn3 fold | unify.rs:1397-409 L07 | pf+verdict br | fold pf into ok | micro | AND->OR fail |
| bn3 emit | unify.rs:1410-420 L07 | gated pushes | pred_inc NEW-KERN | micro | B6 GREEN pattern |
| bn3 chain | unify.rs:1394-422 L07 | chain walk | KEEP | 0 | none |
| bn3 L08 | unify.rs:1340,1518 L08 | sort/seen/linear | NO-SUBSTITUTION | 0 | hit&&=, push muts |
| dred keep | dred.rs:70-74 L09 | 64-iter loop | swar_nonzero mask | 64->2 | FA5 battery |
| dred bfs | dred.rs:383-397 L09 | edge enqueue br | pred_inc | ANALYTIC | FA5 9999 phantom |
| git itoa | pack_write.rs:131-7 L10 | to_string hdr | SWAR LUT itoa | LOW-MED | digit flip; fsck |
| git no | pack_write.rs L10 | no NUL scan | NO-TARGET | 0 | roundtrip |
| git tree | tree.rs ABSENT L11 | not impl GS4 | DESIGN SWAR cmp | ANALYTIC | swap must flip |
| git idx | pack_write.rs:269 L11 | sort_by_key | NO-SUBSTITUTION | 0 | OID shuffle |
| git look | idx_read.rs:138 L11 | 16-stage ladder | eytz REALIZED | 0 | const-iter 466 |



## 2. Top 5 High-Impact Reuse Targets

Ranked by (likelihood-live) x (mechanism-value), ordered on lane evidence:
existing courts, proven byte-equal kernels, and real residual loops outrank
TARGET-only claims.

1. **REP5 penstock wire (gate)** — lane-01 S2+S5+S6. `penstock_prefilter8` is
   a drop-in with identical semantics per dispatch, and the variable-length
   zero-detect directly replaces `is_zero_at` (penstock.rs:651-660) plus the
   inline digest scan (:550-558). Likelihood-live is the highest in the wave:
   Mutant D (corrupt mix constant `0x2544_5491_4F6C_DD1D`) is an
   already-specified callee-identity court, and Mutant B covers both zero
   sites with one flip. [ANALYTIC] until REP5's receipt measures it.
2. **BR7 radix_bucket visibility unification + pred_inc new-kernel emission
   work** — lane-07 §1+§2. The bn3 private `radix_bucket` (unify.rs:825-828)
   is byte-equal to `cgap-branchless::radix::radix_bucket_12`
   (radix.rs:45-48) with B5's differential court already pinning equality
   (radix.rs:121-142); the wall is visibility only. Mechanism-value: one
   kernel, one court, O(N) fewer divergent copies (N-implementations law).
   The pred_inc emission rewrite at unify.rs:1406-1420 is new-kernel work,
   registered separately, micro gain, verdict branch stays.
3. **BR9 SWAR alt-support lane primitive + pred_inc BFS enqueue (storage)** —
   lane-09 §1+§2. `keep_mask` (dred.rs:70-74) is the one residual per-lane
   scalar loop on the static hot path (64 branchy iterations/word); the
   proposed `swar_nonzero_u32x2_mask` sits on the FA5-proven
   data-independent kernel family (`swar_all_zero_vl` at
   cgap-branchless/src/penstock.rs:64). The BFS enqueue predication via
   `pred_inc` (select.rs:60) rides the same B6 GREEN witness pattern.
   Cascade fixpoint already landed (dred.rs:321,417,437) — wiring only.
4. **BR10 SWAR decimal-itoa headers (git)** — lane-10 #1. The only
   actionable lever on that surface: `git_object_oid` (pack_write.rs:131-137)
   formats `<type> <len>\0` via `to_string()`; SWAR 2-digit LUT itoa removes
   fmt machinery + allocs per object on a SHA-1-dominated path. Gain class
   LOW-MEDIUM, court is the existing pack checksum / `git fsck` /
   delta_refusal courts (digit-flip mutation).
5. **BR6 P1/P2 batched-capacity encode emits (parquet)** — lane-06 P1+P2.
   `push_byte` (mmap_writer.rs:359-372) checks capacity per lane byte and
   `push_varint` (:335-357) checks per varint byte; P1 batches to one check
   per 8-byte group with a checked fallback, P2 hoists the varint check via
   the `varint_len` mirror (:329). Typed refusal preserved (fallback still
   refuses). MB7 TARGET: seal 16.25 ms -> ~13 ms @1M — unmeasured, so this
   ranks below the proven-kernel items despite the largest claimed delta.

## 3. Verification standard: B6 census -> workspace-wide call-site court

The B6 census court (waves/w6-b6-census-court.md) proves isolated-symbol
codegen. Four documented holes must close as it expands:

- **Per-call-site cells keyed by consumer fn.** FA6 (forensic-lane-06.md S5)
  witnessed the gap: `#[inline]`/`#[inline(always)]` bodies at uncensused
  call sites are invisible — `padded_at` (eytzinger.rs:125) carries 2
  branches as a symbol, and branches inside inlined copies at uncensused
  callers (eytzinger_search: 5 flow instrs; to_eytzinger_layout: 9) never
  appear. The expanded court adds one cell per CONSUMER function
  (e.g. `penstock::ingress`, `dred::keep_mask`, `bn3::unify` probe loop),
  so an inlined branchless primitive inside a branchy consumer is caught at
  the consumer, not the primitive.
- **cargo-test blindness fix.** FA6 witnessed cargo test exit 0 while the
  standalone census exits 1 (0 census tests executed under cargo test). The
  census is a standalone binary with its own `fn main`
  (branchless_census.rs); the workspace gate must run the census binary
  directly, and its exit must gate — a `[[test]] harness = false` stanza is
  the cargo-visible form if it must live in `cargo test` (B1's Cargo.toml,
  not the court's file).
- **RVC mnemonic gap fix.** The riscv32 `is_flow` classifier matches an
  exact mnemonic list (beq/bne/blt/bge/bltu/bgeu/jal/jalr)
  (branchless_census.rs:18-19 vs 219-228) and would MISS RVC compressed
  branches (c.beqz/c.bnez/c.j/c.jal) and b*-alias forms (bltz/bgez/blez/
  bgtz). Zero RVC branches in this run's objects is an accident, not
  coverage. The fix: classifier by prefix class (`c.b*`, `c.j*`, `b*` with
  alias expansion) or a mnemonic regex, with a self-check that plants an
  RVC branch and proves the court refuses ([VACUOUS]-class flip).
- **link-dead-code is load-bearing.** Without `-C link-dead-code=yes` the
  object contains 0/11 censused symbols — unused `#[inline]` fns are
  dropped and every cell reads MISSING (witnessed twice: B6 receipt and
  FA6 S2, which confirmed fail-closed behavior). Keep the flag pinned in
  the court's rustc invocation; a rebuild without it is not a pass, it is
  a different (vacuous) subject.
- **Hot-path-only assertion discipline.** Only consumer fns on hot paths
  carry branch-free assertions; typed-refusal cold paths stay branchy by
  design (lane-01 MUST-STAY, lane-02 COLD verdicts). Cold sites get
  REFERENCE-DRIVER cells instead: the deliberately-branchy
  `naive_if_cmov` pattern — a reference fn asserted BRANCHY so the court
  proves it can flag at all. A court that cannot flag BRANCHY is VACUOUS
  (witnessed: neutralized reference -> `[VACUOUS]` exit 1).

## 4. Dead-library close-out law

FA2/FA6 verdict (re-confirmed by BR2 and BR11): `saturated_subs_p28`,
`poison_mask`, `swar_lt`, `swar_splat`, and the eytzinger machinery have
ZERO production callers (FA6: zero consumers of cgap-branchless; the gate
dep is dead). Consume-then-census:

1. A primitive may not claim a census PASS while unconsumed — the census
   judges isolated symbols (FA6 S5), so a green cell on a dead symbol
   proves nothing about production codegen. First a consumer (Top-5 items
   1-3 wire real callers), then the census cell becomes meaningful.
2. `swar_splat` is worse than dead: it was stripped even WITH
   link-dead-code=yes (FA6 S-table) — it has no standalone emission at this
   opt level, so its census cell can never be meaningful; retire or give it
   a caller before counting it.
3. No census PASS for unconsumed symbols: the expanded court marks
   unconsumed primitive symbols as UNCONSUMED (not PASS), exit 1 if any
   production-facing claim rests on them. Loud, never silent.
4. Sequencing law: consume-then-census. Implementation lanes land the
   consumer diff first; the census cell flips to PASS only when the
   consumer is on the tree and its cell is green.

## Receipt tail (this session's reads)

| command | exit |
|---|---|
| `ls explore/ && wc -l branchless-reuse-lane-*.md` | 0 |
| `cat branchless-reuse-lane-01..06.md` (loop) | 0 |
| `cat branchless-reuse-lane-07..11.md` (loop) | 0 |
| `sed -n 1,60p /Users/sac/cgap/docs/jira/v26.11.2/waves/w6-b6-census-court.md` | 0 |
| `sed -n 60,120p .../w6-b6-census-court.md` | 0 |
| `sed -n 100,160p /Users/sac/cgap/docs/jira/v26.11.2/adversarial/forensic-lane-06.md` | 0 |
| `grep -rln "FA6\\|saturated_subs_p28\\|blindness..." waves/` | 0 |
| `grep -n ... forensic-lane-06.md` (mnemonic/link-dead-code lines) | 0 |
