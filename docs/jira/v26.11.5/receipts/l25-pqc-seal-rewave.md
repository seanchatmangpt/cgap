# L25 — pqc_seal mutation re-wave (post-W3)

Standing: PASS

Supersedes the INDECISIVE verdict registered in
`docs/jira/v26.11.5/receipts/l14-pqc-seal.md` (§ mutation, lines 77-90).

## Witnessed this session (lane L25, 2026-11-05)

Baseline: `cargo test -p cgap-crypto --test pqc_seal_courts` green at
HEAD `92fa5f0b` (mldsa substrate committed: f4af9188 + cce95c35
lineage).

Mutant (one-sided domain-byte drop on the pqc_open side, /tmp sandbox,
SHIM-marked): flips **3/6 courts** —
`roundtrip_across_sizes_and_aads`,
`domain_separation_and_nonce_binding`,
`aead_tag_tamper_is_aead_mismatch` (lawful downstream effect: with the
domain dropped, open's signature check fails first, so the tamper court
observes BadSignature where it demands AeadMismatch).

Verdict: **no fence defect** — the anti-vacuity mutant is decisive.
Restore verified byte-exact: working tree clean for
crates/cgap-crypto/, sha256 `d830b42e…` over the committed subject,
re-verified by the coordinator after the lane's 429 death.

## Deviation disclosure

The lane died on API 429 after the restore but before minting this
receipt; the coordinator re-derived the restore state on disk and
minted this receipt from the lane's witnessed court output (its final
report quoted the 3 court names + failure classes). Court counts are
lane-witnessed, not coordinator-re-run: PARTIAL receipt provenance,
PASS standing rests on the lane's quoted output + the on-disk restore
witness.

## Status

The L14 receipt's registered re-wave obligation: CLOSED by this
receipt.
