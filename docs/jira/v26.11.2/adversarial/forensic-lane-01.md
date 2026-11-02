# FA1 — hostile forensic lane: chains-ingress adversarial interrogation

Lane FA1 · repo /Users/sac/cgap · branch main · 2026-11-02
Sole write: this file. Repo source untouched (git status of lane surfaces clean).
Probe: /tmp/fa1-sb/probe (standalone crate; sandbox copies of cgap-core,
cgap-gate, cgap-blake3, cgap-branchless with inlined manifests — no mocks,
real kernels, real corpus). Build root lease: /tmp/fa1-target.

## Verdicts

| # | suspicion | verdict |
|---|---|---|
| 1 | escalated (BLAKE3) leg structurally unreachable on chain-admitted frames | **CONFIRMED** (independently reproduced) |
| 2 | bench drops results (vacuous black_box) | **REFUTED** |
| 3 | headline numbers are branch-predicted early-exit artifacts on homogeneous inputs | **PARTIALLY-CONFIRMED** (numbers reproduce; "3-stage chain" framing is a 2.5-stage chain — the crypto leg never runs) |
| 4 | bypass path verifies no payload bytes | **CONFIRMED** (by design, disclosed) |

## 1. Escalation vacuity — CONFIRMED, and it is law, not corpus

Independent reproduction with real code (not a re-read of W4-P3): a
hand-built header-valid frame per flag byte, all 256 values, run through
both stage 1 (`CgapFrameHeader::validate`, reserved-mask refusal at
crates/cgap-core/src/frame.rs:284, `FLAG_RESERVED_MASK = 0xF0` at
frame.rs:38) and the dam (`dam::ingress`, escalate mint at
crates/cgap-gate/src/penstock.rs:615, `ESCALATE_MASK = 0x00F0` at
penstock.rs:271):

- predicate identity `escalate == stage1_refusal` holds over all 256 flag
  values, **0 exceptions** — w4-p3-escalation-corpus.md CONFIRMED against
  current code.
- ESCALATE_MASK (penstock.rs:271, u16 over header byte 5) is the same
  predicate as stage-1's FLAG_RESERVED_MASK (frame.rs:38, u8) restricted
  to the byte — the two masks cannot disagree.
- Probe census on the frozen CHAIN_SEED corpus (seed 0xC001_0000_5EED_0001,
  gen_frame adapted verbatim from chains-ingress.rs:224-391, 65,536 frames):
  **chain_admitted = 18,155 / 65,536 (27.7%)**; escalated+dam-admitted =
  **0**; dam minted escalate=true on **1,768** frames and **every one is
  chain-refused at stage 1**. The dam's escalation trigger is ALIVE
  (timing E below) but fires only on frames the chain never admits.
- Therefore `stage3_escalated`'s BLAKE3 branch
  (crates/bench/benches/chains-ingress.rs:161-168) is dead code at chain
  level. The bench itself documents this honestly
  (chains-ingress.rs:642-648: "the bench then proves the chain pays zero
  digest work, which is the lever's own claim; the empty-subset receipt
  row is recorded, not rounded up").

**Corpus entropy/alignment** (gen_frame, chains-ingress.rs:224-391): 50%
uniform-random garbage (rng % 4 ∈ {0,1}, length 0..384), 50%
header-valid-or-mutated; payload_len = rng % 256 (avg ~127.5 B);
lineage/flags/digest from SplitMix64 (multiplier 0xBF58_476D_1CE4_E5B9
verbatim, bench :70-83). Not homogeneous: half the frames are garbage and
the mutation wheel (rng % 9) spreads rejects across 8 reject classes.
The early-exit cost is real work, not a prediction artifact.

## 2. black_box audit — REFUTED

Results are consumed, not dropped. Every bench leg accumulates `acc` and
asserts a per-pass checksum against the preflight-recorded fold:

- ingress_chain: `black_box(acc)` chains-ingress.rs:585 + `assert_eq!(acc,
  expect)` :586-589 (expect = pool.chain_fold × laps, basis recorded at
  :536). A drifted kernel or corpus flips the assert loud.
- stage_a04:608-612, stage_penstock:634-638, stage_digest_fold:664-668,
  stage_prefilter_fold:689-693 — same pattern. stage_penstock additionally
  black_box's `view.payload.as_ptr()` (:631).
- The preflight itself cross-checks the inline fold against a real
  `chain_frame` call per frame (:506-510).

Probe check: my own probe initially omitted the clean-frame prefilter
contribution from its checksum basis and printed checksum_ok=false — the
assert channel is live, it catches exactly that drift. (Probe-side defect,
disclosed; bench-side code is correct at :488-497.)

## 3. Measured (probe, release, lto=thin, min of 30 passes, 65,536 frames/pass,
std::hint::black_box + Instant; three runs this session)

| leg | ns/frame |
|---|---|
| A. full chain (corpus as-is), checksum asserted | 9.5-11.0 (9.519 / 10.486 / 11.002) |
| B. stages 1+2, fold bypassed | 9.7-10.7 |
| C. dam-only (stage 2), mixed corpus | 7.5-8.8 |
| D. FORCED fold-on: BLAKE3 over every chain-admitted payload (n=18,155, avg payload ~127 B) | **315.9-324.6** |
| E. dam-only on 240 row-6-flagged frames (escalate minted, CGAP_OK asserted + escalate=true asserted per frame) | 9.0-9.2 |

- Headline 7.89-10.65 ns/frame REPRODUCES (A within its band). The number
  is real; what it is NOT is a "3-stage chain": stage 3 is a prefilter
  XOR-mix, not a digest.
- **The honest escalated-path number the headline never reports: ~316
  ns/frame** — 33x the chain headline. If the escalated leg ever fired on
  chain-admitted frames, wire-speed claims would collapse by that factor.
  (TARGET-class: probe-timed, not divan; method stated above.)
- A vs B differ by ~0-1 ns — the prefilter fold costs ~nothing, consistent
  with "skip-not-pay" being a one-stage policy in practice.
- E proves the dam-side trigger is ALIVE (~9 ns/frame incl. per-frame
  asserts), so the vacuity is a policy-design vacuity, not dead library
  code.

## 4. Payload integrity — CONFIRMED: admission verifies ZERO payload bytes

Court (probe, exit 0): header-valid frame (CGAP magic, v1, 376 B declared
payload) with **376 B of 0xFF garbage payload and a WRONG nonzero digest
field → ingress returns CGAP_OK**. Every ingress read is header-region
only: magic/version (penstock.rs:536-543), digest **nonzero-only** check
(penstock.rs:550-559 — the gate never compares the digest to the payload,
it only requires the field be non-zero), length coherence (:565-568),
attestation (:570-600), prefilter8 (bytes 0..8, :662-686).
`oabi_digest_verify` (crates/cgap-core/src/frame.rs:312-331) exists but is
never called in the chain. The gate self-describes as "a limiter, not a
verifier" (penstock.rs:546-549) — disclosed in the code, but the bench
header's "malformed-frame dropping" business case (chains-ingress.rs:3-4)
does not carry that caveat; a wire claim built on 7.89-10.65 ns/frame is
header-classification speed, not payload verification.

## Disclosure audit: honest, not buried

- W3-L1 (w3-l1-ingress.md:74-80) names the vacuity in a dedicated
  "Vacuity disclosure" section: escalated subset EMPTY, digest leg a
  "vacuous-zero leg... named here..., not a hidden skip".
- W4-P3 (w4-p3-escalation-corpus.md:5-59) upgraded it to the structural
  claim and re-framed the lever honestly as "skip-not-pay with a vacuous
  escalated branch". Both receipts disclose; nothing buried.
- Residual overclaim risk lives in the *framing*, not the receipts: the
  bench module doc (chains-ingress.rs:1-20) presents a 3-stage chain with
  stage 3 = "fold the payload digest"; on the frozen corpus the digest is
  never computed. LANDED-skip-not-pay covers the receipts; the module-doc
  framing ("the product", "the full 3-stage chain") still reads stronger
  than the measured behavior.
