# aoq-threat-model

AOQ1 wire law — threat model and guarantees register. Provenance note:
AF-17's in-session report was not on disk at AF-43 time (2026-11-04); this
document was reconstructed from the LANDED receipts
(`docs/jira/v26.11.3/receipts/ab-2..ab-8`, `cg-1`, `af-10`) against the
landed source in `crates/cgap-aoq/src/`. The section skeleton (5 assets,
blind-routing premise, 4 adversaries, Gap-1, 5 non-guarantees, OQ-1..5)
is the AF-43 contract; per-item content is grounded in the cited
receipts. Open items are UNVERIFIED until AF-17's owner reconciles this
reconstruction against the in-session report.

## Assets (5)

| # | Asset | Grounding |
|---|---|---|
| A1 | Frame confidentiality: AEAD-protected payload under a per-epoch key | ab-4-aead, ab-3-chacha |
| A2 | Frame authenticity/integrity: Poly1305 tag + AoqChain digest chain | ab-3, ab-7-chain-session |
| A3 | Epoch key material: ML-KEM-768 capsules and epoch keys; decaps infallible-by-type | ab-5-mlkem |
| A4 | Sequence/order state: wrap-safe successor law, SeqGap typed refusal | cg-1, ab-6-header |
| A5 | Refusal fidelity: typed refusal classes downstream policy reads; forgery would misroute lawful traffic | ab-6-header, af-10 |

## Blind-routing premise

Middleboxes forward AOQ1 frames by header metadata only (wireKind, epoch,
sequence) without possessing epoch keys; the header is plaintext by design
so routers classify without key possession. Consequence 1: the header is a
public attack surface — every header field must survive full mutation
coverage (aoq1_stress_court: 10k single-byte mutations of a valid sealed
header all land in typed variants, no silent acceptance). Consequence 2:
payload confidentiality/authenticity never depend on header secrecy; they
depend on the AEAD tag and the digest chain, which the header cannot forge.

## Adversaries (4)

| # | Adversary | Capability | Held by design |
|---|---|---|---|
| W1 | Passive eavesdropper | Reads all wire bytes | A1 holds; header is plaintext BY DESIGN (see premise) |
| W2 | Active tamperer (Dolev-Yao on the wire) | Reads, drops, reorders, mutates in flight | A2/A4 hold; tamper lands in typed refusals (aoq1_stress_court full mutation coverage) |
| W3 | Frame-injection adversary | Crafts frames from the public header law | Holds only if it forges the AEAD tag or chain digest; I4 census: 12,000 tampered frames, per-class census exact |
| W4 | Harvest-now-decrypt-later | Records today, decrypts after a quantum break of classical transport keys | ML-KEM-768 epoch capsules (A3) close the HNDL channel for payloads; Gap-1 qualifies this |

## Gap-1: KEM identity binding

ML-KEM-768 `ek` carries no identity binding: an encaps-to-`ek` step cannot
distinguish the intended peer's encapsulation key from a substituted one.
Absent an out-of-band binding (signature over `ek`, certificate, or
pre-shared binding), a substituted `ek` yields frames the substitutor can
decrypt — the classic KEM identity-misbinding class. The landed kernel
computes `encaps(ek, m)` with no identity input
(`crates/cgap-crypto/src/kem.rs`; AB-5 kept decaps infallible-by-type, no
RNG in-crate). This gap is REAL (landed-code-grounded), qualifies assets
A1/A3, and is the top open question (OQ-1).

## Non-guarantees (5)

The wire law does NOT promise:

1. No peer identity authentication. No binding layer exists; A1/A2
   authenticate keying material, not parties (Gap-1).
2. No DoS resistance. Tamper floods cost CPU; refusal parity is fenced at
   1.5x (I4), but no rate limiting or proof-of-work exists.
3. No traffic-analysis resistance. Frame size, timing, and header metadata
   are public by the blind-routing premise.
4. No forward secrecy across epochs. Per-epoch isolation holds; compromise
   of an epoch key exposes that epoch's frames, not others'.
5. No replay protection beyond the chain window. SeqGap detection covers
   the receive window; frames outside the window are indistinguishable
   from fresh ones (bounded by the slab lease).

## OQ register (OQ-1..5)

Open questions carried to the AF-17 owner for reconciliation. Only OQ-4
has a disk receipt (ab-5:98); the others are UNVERIFIED reconstructions
from landed code.

- **OQ-1** — Does a binding layer (signature over `ek`, certificate, or
  pre-shared binding) land, and in which crate? (Gap-1 closure path.)
- **OQ-2** — Is header plaintext exposure acceptable for all
  wireKind/epoch combinations, or does any wireKind need header protection?
  (Blind-routing premise scope.)
- **OQ-3** — Are the SeqGap receive-window bounds correct for production
  lease durations? Wire bounds are receipted (FrameTooLarge at 4080,
  ab-8-wire); window-exhaustion behavior is UNVERIFIED.
- **OQ-4** — ML-KEM-768 rekey-cost baseline: measured for one target
  (ab-5:98), UNVERIFIED for others.
- **OQ-5** — Refusal fidelity under concurrency: the I4 parity fence is
  measured single-lane; concurrent-lane refusal parity is UNVERIFIED
  (af-32 saturation runs parallel lanes but does not fence refusal parity).

## See Also

- `docs/jira/v26.11.3/receipts/ab-2..ab-8` — AoQ kernel receipts
- `docs/jira/v26.11.3/receipts/af-10-cbmqs.md` — CBMQS gate map
- `crates/cgap-aoq/src/` — landed kernel surface
