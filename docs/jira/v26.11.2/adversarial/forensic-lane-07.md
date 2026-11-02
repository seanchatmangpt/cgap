# Forensic Lane FA7 — footer.rs seal-path capacity interrogation

Lane: FA7 (hostile forensic). Subject: `crates/cgap-parquet/src/footer.rs`
(P5 MetaBuf fixed scratch + `[u32; MAX_ROW_GROUPS]` spans + typed refusals).
Only repo write: this file. Probe source: `/tmp/fa7-probe` (path-dep on
cgap-parquet); build root `/tmp/fa7-target` (lease, deleted at lane end).

## 1. Constants and refusal paths (read from source)

- `META_CAP = 512` (footer.rs:89), `MAX_ROW_GROUPS = 64`
  (`mmap_writer.rs:72`), `ATTESTATION_LEN = 80`,
  `TRAILER_FIXED = 96`.
- Every `MetaBuf` growth site bounds-checks: `push` (footer.rs:114-121)
  and `extend` (footer.rs:123-130) return typed
  `ParquetError::RunOverrun { run_len, budget }`; there is no unchecked
  write into the scratch. The `spans` `copy_from_slice` (footer.rs:490) is
  guarded by the `row_groups > MAX_ROW_GROUPS` refusal (footer.rs:441-450)
  and the `spans.len() == row_groups` refusal (footer.rs:451-456), so the
  inline array cannot be overrun.
- `seal()` computes the end extent with `checked_add` chains
  (footer.rs:516-529) and refuses BEFORE the first buffer byte is written
  (footer.rs:530-535) — no partial overwrite on any refusal path.

## 2. Adversarial probe arms (command + exit this session)

`/tmp/fa7-target/release/fa7-probe; echo "PROBE EXIT $?"` — exit 0, no
panic, no segfault, any arm.

| Arm | Input | Observed | Verdict |
|---|---|---|---|
| 64 RGs (exactly MAX_ROW_GROUPS) | 64 spans of 3, body 192 B | seal Ok, total=384, footer_len=192 | PASS |
| 65 RGs | 65 spans | `RunOverrun { run_len: 65, budget: 64 }` | typed refusal, exact payload |
| 10,000 RGs | 10000 spans | `RunOverrun { run_len: 10000, budget: 64 }` | typed refusal, no panic |
| span = i32::MAX+1 | 1 span | `RunOverrun { run_len: 2147483648, budget: 2147483647 }` | typed refusal (W7-F1 check live) |
| 8 RGs x 256 MiB spans | 2 GiB body | seal Ok, m=61 B, meta byte-exact vs probe's independent Vec oracle (full stream) | PASS |

**FINDING FA7-D1 (real defect, uncourted): rows >= 2^63 seal-but-refuse
asymmetry.** `FooterSealer::new` checks body/row_groups/spans/sum but NOT
`summary.rows`; `seal()` emits `zigzag_i64(self.rows as i64)`
(footer.rs:579) — for `rows >= 2^63` the `as i64` cast wraps negative, so
the sealer emits a block that `parse_meta` refuses with
`Shape(MetaMalformed)` at the `rows < 0` check (footer.rs:867).
Witnessed this session:

```
ARM7 rows=0x7fffffffffffffff: seal Ok, verify -> VERIFIED rows=9223372036854775807
ARM7 rows=0x8000000000000000: seal Ok, verify -> Err Shape(MetaMalformed)
```

This is the W7-F1 class (writable-but-unverifiable block) at a THIRD
bound (rows), exactly the pattern W7-F1 closed for spans — but on rows,
with no court. All 12 in-crate footer courts pass (`cargo test -p
cgap-parquet --lib footer::`, exit 0) while the defect is live: the
differential court's fixtures use `rows = 0xABCD_1234_5678_9A00` (top bit
set!) for byte-compare only and never run `verify_seal` on them; round-trip
courts use small rows. Suggested repair: a
`rows > i64::MAX` typed seal refusal in `FooterSealer::new` + a court
pinning rows = i64::MAX+1 (exact payload `run_len = rows, budget =
i64::MAX`), mirroring the W7-F1 span court.

## 3. Byte-exactness court scope

The differential court (`meta_fixed_path_is_byte_exact_vs_vec_oracle`)
compares the FULL byte stream: `assert_eq!(&scratch[..m], &expected[..])`
— slice equality includes length, so a prefix-only drift detector it is
not. Independently re-witnessed this session by the probe (its own Vec
oracle built from the crate's public `write_*` family): full-stream exact
at 64 RGs and at 8 RGs x 256 MiB spans. Not a prefix compare.

## 4. META_CAP sizing and the bounded-input question

Worst-case lawful meta at MAX_ROW_GROUPS: f1=2 B, f2=3 B, f3=11 B (max
zigzag-i64), f4 = 3 B headers + 64x5 B spans = 323 B, f5=10 B, f6=2 B,
STOP=1 B -> 352 B <= META_CAP=512 (160 B headroom). So 64 RGs x worst-case
spans fits by construction; 10K RGs is refused upstream at the
MAX_ROW_GROUPS check before the scratch is touched.

META_CAP+1 is NOT publicly drivable: meta size grows with span MAGNITUDE
(5-byte zigzag needs span >= 2^28, i.e. >= 256 MiB body per span), so
exhausting 512 B of scratch needs ~64 max-magnitude spans -> a ~17 GB
body, not allocatable in-process. The typed exhaustion path itself is
real and is exercised by the in-crate court that drives
`build_meta_block` against the META_CAP scratch directly. Probe witness:
8 spans of 256 MiB produce only 61 B of meta.

**Disclosure verdict: PARTIALLY disclosed, not buried.** Code docs state
it plainly (footer.rs:85-88: "worst-case §6.2 encoding at
MAX_ROW_GROUPS... Overflow is a typed refusal, never a grow"). The w6
receipt names the 512-byte scratch and the >64-RG refusal but never states
the bounded-input arithmetic or the guarantee's scope — the "zero-alloc
seal" headline is accurate only as a bounded-input guarantee (bounded
row-group count AND bounded span magnitude), and the receipt's honest
notes don't say so. Not buried, but the receipt under-discloses scope.

## 5. Verdicts per suspicion

1. "Hardcoded static buffer silently truncates or corrupts on large
   metadata" — REFUTED: every growth site is a typed bounds-checked
   refusal; probe arms confirm no truncation, no panic, no overwrite.
2. "Differential court is a prefix compare" — REFUTED: full-stream
   compare, externally replicated.
3. "Capacity bounds refuse typed at every boundary" — PARTIAL: RG/span
   bounds all refuse typed with exact payloads, but **rows >= 2^63 is an
   uncourted seal/verify asymmetry (FA7-D1)** — the adversarial
   presumption is vindicated one level over: not truncation, but a
   writable-but-unverifiable block at the rows bound.
4. "META_CAP sized for realistic, refuses absurd" — CONFIRMED with the
   arithmetic above (352 B worst case in 512 B, 10K RGs refused upstream).
5. "Bounded-input scope disclosed" — PARTIAL: code docs yes, w6 receipt
   under-discloses.

## 6. Commands + exits

- `/tmp/fa7-target/release/fa7-probe` — exit 0 (7 arms incl. rows
  isolation; no panic anywhere).
- `CARGO_TARGET_DIR=/tmp/fa7-target cargo test -p cgap-parquet --lib footer::`
  — exit 0, 12 passed / 0 failed (suite green while FA7-D1 is live:
  coverage hole, not a regression).
- Shared tree untouched except this file; build root /tmp/fa7-target
  (lease deleted at lane end).

## Standing

ALIVE for every table row and FA7-D1: command + exit run this session
against the exact subject. FA7-D1 is UNVERIFIED-as-repaired (no repair
attempted; out of lane scope — read/write = this file only).
