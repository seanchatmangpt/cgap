# DOC-3 moonshot-a README receipt

Lane: DOC-3 (lane-runner). Date: 2026-11-01. Repo: /Users/sac/cgap.
Lease honored: five moonshot-crate READMEs + readme-archive/ + this file.

## Archive

- crates/cgap-core/README.md (81 lines) archived to
  `docs/jira/v26.11.1/readme-archive/cgap-core.md`, sha256
  `00cdbe9c8e5eb214b187b4fe25706c3bf9b585c7776f7854fa243f495841e97a`.
- cgap-storage / cgap-bont / cgap-bosym / cgap-bn3 had no README:
  nothing to archive (noted, not a skip of real content).

## crates.io state (witnessed 2026-11-01, UA cgap-publish-receipt)

| Crate | API verdict | Evidence |
|---|---|---|
| cgap-core | NOT PUBLISHED (HTTP 404) | curl exit 0, body 404 |
| cgap-storage | NOT PUBLISHED (HTTP 404) | same |
| cgap-bont | PUBLISHED-26.11.1, num_versions=1 | API fields |
| cgap-bosym | PUBLISHED-26.11.1, num_versions=1 | API fields |
| cgap-bn3 | NOT PUBLISHED (HTTP 404) | curl exit 0, body 404 |

Publish-guard context: all five carry explicit `publish = true`
(waves/p3-pub-guard-flip.md); bont/bosym published one-attempt
(waves/p3-pub-bont-bosym.md); core/storage were UNSUPPORTED-structural
at p3-pub-core-storage.md read time (guard has since flipped; publish
still not witnessed at THIS receipt's read time). Statuses are AS
WITNESSED, never aspirational.

## READMEs written

- crates/cgap-core/README.md (39 lines): 128B frame pin, ingress courts
  75/0, mutation flip E0080.
- crates/cgap-storage/README.md (33 lines): 113-offset power-cut sweep,
  zero torn writes, both-slots + erase sweeps.
- crates/cgap-bont/README.md (31 lines): SlotId law
  (truncate64/BLAKE3, tag b"bont\x01"), real collision-trap pair,
  35/0.
- crates/cgap-bosym/README.md (30 lines): ingress limiter diode,
  14/0, named mutation flip.
- crates/cgap-bn3/README.md (37 lines): 24B statement pin, fixed
  arena, 111/0, 2/2 sandbox flips.

## Verification

- awk >100-char sweep over all five READMEs: zero hits.
- wc -l: all five <= 39 lines (contract cap 80).
- No git commands run (contract). Numbers cited from waves/ receipts
  and live API reads above; nothing invented.

## Standing

ALIVE for the five READMEs + archive + this receipt: every number
cited carries a receipt pointer or a this-session command. Divergence:
bont/bosym READMEs point at waves/a06-a07 receipts; the a06 path
cited in crates/cgap-bont/README.md names both lanes/ and waves/
copies (waves/ copy is the one read this session).
