# CGAP-PRD-2 audit: Chatman Equation symbol grounding
Lane: CGAP-PRD-2 | 2026-11-02 | ledger-honesty trace
Target: docs/jira/v26.11.2/12-ard-prd-cgap.md:19 prints:
  `ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC)`
Every symbol was traced to code. Three land as typed commit
trailers; one term has NO referent anywhere in the tree.

## Verification commands (fresh, this audit)
1. grep -n "KEY_LEASE_DELTA" \
     crates/cgap-git/src/commit_synth.rs
   -> consts land at lines 60-62.
2. grep -rniE "sigma[ _-]?pqc|SigmaPQC" \
     --include="*.rs" crates/
   -> ZERO hits. No Rust referent, any crate.
3. grep -rn "Sigma_PQC" docs/ crates/
   -> prose-only hits (7 docs), zero code hits.
4. grep -n "blake3" crates/cgap-crypto/src/chatman.rs
   -> digest machinery is BLAKE3 throughout.

## Symbol / referent / file:line / honest status

### ΔS_admitted
- referent: KEY_LEASE_DELTA = "lease-delta " commit trailer;
  value is a u64 admitted-delta count, rendered by push_u64
- file:line:
  crates/cgap-git/src/commit_synth.rs:61,188-189,262-263
- honest status: REAL, arithmetic false. It is a counted u64
  trailer, not a term summed modulo anything.

### Φ_lease
- referent: KEY_LEASE_PHI = "lease-phi " commit trailer;
  32B hex, caller-supplied via CommitSpec.lease_phi; the
  module never computes it — render emits, never derives
- file:line:
  crates/cgap-git/src/commit_synth.rs:60,187,260-261
- honest status: REAL as an opaque pass-through digest.

### R_receipt
- referent: KEY_LEASE_RECEIPT = "lease-receipt " trailer;
  LeaseReceipt = nonce u64 LE + parent 32B + payload 32B +
  tag 8B = 80B, domain-separated tag, no padding holes
- file:line:
  crates/cgap-git/src/commit_synth.rs:62,111-156,265-267
- honest status: REAL. 80-byte framing exists; sealed and
  verified fail-closed (ReceiptTagMismatch refusal).

### Σ_PQC
- referent: NONE. Zero code referent in cgap-ffi,
  cgap-crypto, cgap-p28, or any crate in the tree.
- checked via: grep command 2 above (zero .rs hits)
- context: the digest is BLAKE3 (classical, not
  post-quantum); the PQC seal is an unlanded Wave-3 seam
  (crates/cgap-crypto/src/chatman.rs:9-10,351,368,422)
- honest status: INVENTED LORE. "mod Σ_PQC" exists in prose
  only; no modulus object, no code path, anywhere.

## Docs repeating Σ_PQC as landed fact (all prose-only)
- docs/jira/v26.11.2/12-ard-prd-cgap.md:19
- docs/jira/v26.11.2/paper/position-paper-v3.md:231,233
- docs/jira/v26.11.2/paper/claims-appendix.md:37
- docs/jira/v26.10.31/07-chain-economy-8020.md:36
- LICENSE-P28-SIL.md:69
- docs/REVENUE_GENERATION_COMPENDIUM.md:103
- docs/jira/v26.11.1/lanes/A16-chatman-pqc.md:28,97-99
  (A16 alone is honest: it flags `mod Sigma_PQC` as an
  UNKNOWN, not landed fact. The other six restate it as law.)

## Exact rewrite of the equation block
Replace the equation block with:

    The Chatman Equation's landed trace is three typed commit
    trailers with typed refusals — `lease-phi`, `lease-delta`,
    `lease-receipt` — and a fail-closed render; no modular
    arithmetic exists. `lease-phi` is a caller-supplied 32-byte
    digest the render passes through; `lease-delta` is a u64
    admitted-delta count; `lease-receipt` is an 80-byte frame
    (nonce || parent || payload || tag) verified fail-closed
    against its domain-separated tag. There is no Σ_PQC term in
    the tree: the digest is BLAKE3 (classical), and any
    post-quantum seal remains an unbuilt Wave-3 seam.

## Bottom line
3 of 4 symbols are receipt-true. Σ_PQC is not deferred
engineering — it is a symbol with no referent, restated as a
conservation law by six docs. Keep the trailers; drop the
modular-arithmetic framing; cite A16's UNKNOWN as the only
honest prior statement.
