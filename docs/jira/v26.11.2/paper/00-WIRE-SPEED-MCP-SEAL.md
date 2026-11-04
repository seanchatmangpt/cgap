# 00-WIRE-SPEED-MCP-SEAL

Seal verdict for the "Zero-Copy Agency" paper, minted 2026-11-03 by the
coordinator (session continuation of wf_22d132b9-dc2), /Users/sac/cgap @
main. This file is the P-20 verdict: it names every receipt the seal
stands on, the seal-blocking conditions that were raised and closed, and
the conditions that would break the seal.

## 1. Sealed subjects (all ALIVE this session, exact on-disk subjects)

| Subject | Receipt | ALIVE evidence |
|---|---|---|
| docs/jira/v26.11.2/paper/zero-copy-agency.md + parts/ (347 lines, byte-identical assembly, 54 receipt-tags / 7 TARGET / 6 CITE-TBD) | ../lanes/lane-paper-assembly.md + ../lanes/lane-paper-repair.md | coordinator cmp re-run + court-auditor re-audit (parts cmp exit 0; line-length 0 non-table) |
| docs/jira/v26.11.2/bench-validate/p01-stdio-baseline.md | ../lanes/ + git untracked receipt pair | measured N=100x3, harness /tmp/p01-harness, replay commands on file |
| docs/jira/v26.11.2/bench-validate/p02-http-baseline.md | same directory | measured N=100 HTTP/1.1 keep-alive legs, layer-boundary fence §1 |
| docs/jira/v26.11.2/bench-validate/p17-token-churn.md | ../lanes/lane-p17-token-churn.md | N=1000 tiktoken o200k_base; 14.06% byte saving / +27.90% token cost; ROUNDTRIP_FAILURES 0 |
| crates/wire-shm (v26.11.2, zero-dep) | ../lanes/lane-wire-shm.md | cargo test 8/8 rc 0 (witnessed twice this session); CRC-poly mutation court FLIPPED |
| crates/cgap-mcp (v26.11.3, publish-intended) | crates/cgap-mcp/RECEIPT.md | cargo test 5/5 rc 0 (witnessed twice this session); sandbox ordinal mutation FLIPPED; real-stdio protocol court |

## 2. Verification gates (P-19)

verification-sweep-verdict.md: PASS, 0 violations (A1-A5, S1-S2, R1-R3,
C1, D1-regression all exit 0 / cmp exit 0 / cargo rc 0).

## 3. Red team (P-18) — raised, repaired, closed

red-team-verdict.md found 2 SPOILER-CONFIRMED (S1 bridge-described-
unbuilt; S2 14.06% misattributed to WIR0) + 5 NITs. Repair lane closed
S1, S2, N1-N4 (receipt: ../lanes/lane-paper-repair.md). Court-auditor
re-audit on the repaired paper: SPOILER-CONFIRMED count 0.

## 4. Known NITs carried on the seal (non-blocking, disclosed)

- N5: coordinator census errata chain (50→45→54) closed in
  ../lanes/lane-paper-assembly.md — final figure 54, re-derived on disk.
- N6: the paper's minimal-subset scope statement cites
  crates/cgap-mcp/RECEIPT.md:24-29, which enumerates the protocol
  surface but does not contain a scope sentence in those words.
  Under-claim direction; content verified true.
- N7: bare `crates/...` receipt cites (paper:186,192,194,206) resolve
  by repo-root convention, not from the paper directory. Content
  verified correct.
- StaleOrdinal coverage: cgap-mcp's own suite does not flip on
  StaleOrdinal-check removal; cross-crate coverage lives in wire-shm's
  `wraparound_reclaims_oldest_stale_read_typed`
  (crates/wire-shm/tests/wire_courts.rs:45-62), which asserts a typed
  error on a genuinely stale read and fails by construction if the
  check is removed. Sandbox re-witness of that flip was permission-
  denied; the flip is argued structurally, standing UNVERIFIED-locally,
  ALIVE-by-receipt (lane-wire-shm.md court table).

## 5. WIR0-specific byte saving

Unmeasured. The paper's 14.06% figure belongs to the p17 harness-local
compact codec (3-B header); WIR0's 16-B header reduces any such saving
on the same corpus by an unmeasured amount. Tagged [TARGET-v26.11.2]
in the paper. Any future claim of a WIR0 saving requires a measuring
receipt against WIR0 itself — this seal does not admit one.

## 6. Token-layer fence (F7)

The +27.90% token cost of binary frames (compact payloads, tiktoken
o200k_base) stands as a measured REFUTATION of token-layer parity.
KB-scale payload token ratios: UNVERIFIED, stated as such.

## 7. Seal-breaking conditions

The seal breaks if any of: (a) a quantitative claim appears in the
paper without a receipt/TARGET/CITE-TBD tag; (b) a WIR0 or cgap-mcp
figure is quoted without a measuring receipt; (c) cargo test fails in
crates/wire-shm or crates/cgap-mcp at a published version; (d) any
[CRYPT-TBD]-class citation ships unpinned; (e) the token-layer
refutation is restated anywhere as a token saving.

## 8. Standing

PAPER SEALED at 2026-11-03 against the exact subjects in §1.
Remaining campaign work outside this seal: Wave 4 (26.11.3 CalVer
bump, gate matrix, crates.io publish of cgap-mcp after wire-shm) and
Wave 5 (submission package; author identity fields REQUIRE-OPERATOR).
