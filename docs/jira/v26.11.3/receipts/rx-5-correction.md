# RX-5 Correction — RFC-0089 Certificate Green-Table Hazard (class-3)

Subject: `/Users/sac/cgap/crates/cgap-aoq/tests/rfc0089_certificate.rs`
(the certificate generator: `scorecard_table()` / `g7_leg_b_from_receipt()`)
plus the new court
`/Users/sac/cgap/crates/cgap-aoq/tests/certificate_gate_table_court.rs`
Standing: ALIVE (ran + passed this session, commands + exits below)
Date: 2026-11-04

## The hazard (class-3)

The §8.4 certificate's gate table (`green_table()`) hand-encoded G7 as
**Pass** while the RFC §6 record (RX-3 scorecard:
`crates/cgap-aoq/tests/rfc0089_scorecard.rs` `gate_g7()` leg B,
receipted in `docs/jira/v26.11.3/receipts/rx-3-scorecard.md`) records
G7 **MEASURED-UNMET**: AB-14's 50k span 1276.01 ms / 50,000 =
**25.5202 µs/link** vs the ≤ 25 µs/link target.

A certificate that marks PASS where RFC §6 records UNMET is a false
qualification statement — the exact drift class the certificate exists
to prevent.

## Fix (single source of truth)

`green_table()` is retired. `scorecard_table()` derives the G7 row by
reading the AB-14 receipt ON DISK
(`/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md`)
— the same file the scorecard's `gate_g7()` leg B reads — and applying
the same §6 law (parse the "50k span" ms value, `ms·1000/50000` µs/link
vs 25.0). Unreadable/unparsable source → `Verdict::Skip`, which
`issue()` already fails closed on (`CertificateIncomplete`) — a
certificate can no longer issue at all if the §6 source is unmeasurable.

- G1–G6 rows are unchanged (receipt-cited rows for ab-6/ab-9/ab-19).
- The honest table is NOT all-green: G7 = `Fail` →
  `Overall::Disqualified`. The tamper court's overall-flip mutation was
  updated accordingly (base is Disqualified; mutation flips to Pass).
- `g7_leg_b_from_receipt(path)` is `pub` so the court can re-derive
  against mutated /tmp receipt COPIES (real receipt only ever read).

## Before / after — G7 row

| | before (hand-encoded) | after (§6-derived) |
|---|---|---|
| G7 verdict | `Pass` | `Fail` (MEASURED-UNMET) |
| G7 measured | "receipt per frame; chain verify green; per-link cost measured" | "RFC §6 G7 MEASURED-UNMET: receipted 50k-span cost 1276.01 ms / 50,000 = 25.5202 µs/link > 25 µs/link target (debug profile, min-of-3, cgap-ffi RingReceiptChain plane; same source the RX-3 scorecard leg B reads)" |
| overall | PASS | DISQUALIFIED |
| source | hand-typed row | AB-14 receipt on disk (same file scorecard leg B reads) |

## New court (`certificate_gate_table_court.rs`)

Includes the generator as a module (`#[path]`), fails if the
certificate table ever disagrees with the §6 source again:

1. `court_certificate_g7_agrees_with_scorecard_source` — reads the RX-3
   scorecard receipt's G7 row (must contain MEASURED-UNMET), derives
   the certificate G7 row, compares against the court's own INDEPENDENT
   oracle parse of AB-14 (deliberately not shared with the generator),
   asserts DISQUALIFIED + seal binds the honest body.
2. `court_g7_derivation_flips_on_mutated_receipt_copies` — anti-vacuity:
   mutated /tmp receipt copies flip the verdict (1200.00 ms → Pass at
   24.0 µs/link; 1300.00 ms → Fail at 26.0 µs/link; min-of-3
   disclosure stripped → Skip); a SKIP-class G7 row refuses the
   certificate typed (`CertificateIncomplete { skipped_gates: ["G7"] }`).
   Real receipt never written; tmp copies deleted at the end.
3. The generator's own 5 courts re-run inside this binary (its
   `#[cfg(test)]` module compiles under the included module) — same
   laws, re-asserted.

## Gates (exit records)

From `/Users/sac/cgap`, this session:

```
cargo test -p cgap-aoq --test rfc0089_certificate
```

Run 1: `test result: ok. 5 passed; 0 failed` — exit 0.
Run 2: `test result: ok. 5 passed; 0 failed` — exit 0.

```
cargo test -p cgap-aoq --test certificate_gate_table_court
```

Run 1: `test result: ok. 7 passed; 0 failed` (5 generator courts + the
2 new court tests) — exit 0.
Run 2: `7 passed; 0 failed` — exit 0.

All 13 compile warnings are pre-existing cgap-crypto lib warnings
(poly1305/kem/fips203), zero warnings from the touched files.

## Diff ownership

Touched exactly: `rfc0089_certificate.rs` (the generator — required by
the correction), new file `certificate_gate_table_court.rs`, this
receipt. `rfc0089_scorecard.rs` and all other existing test files:
untouched. No git operations (lane law; coordinator owns transitions).
