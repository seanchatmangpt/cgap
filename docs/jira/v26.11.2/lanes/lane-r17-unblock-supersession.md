# Lane R-17 — SLA unblock supersession sweep

Repo /Users/sac/cgap @ main, 2026-11-02. READ-ONLY sweep; zero edits made.
No git commands. Build root lease /tmp/r17-target (deleted at lane end).

## Verdict table (7/7, no residue)

| # | File | Unblock (disclosing receipt) | Verdict | Evidence |
|---|------|------------------------------|---------|----------|
| 1 | crates/cgap-p28/src/attestation.rs | keyed_digest import (RC-11 W6.1) | SUPERSEDED | owner import at :1402; no duplicate |
| 1b | same | INPUT_LEN 42→43 const-assert (RC-11 W6.2) | CONSISTENT | :1447-1448, MAC_DOMAIN.len()+16, assert ==43 |
| 2 | crates/cgap-bn3/src/noalloc.rs | duplicate required_bytes removal (RC-11 W6.3) | SUPERSEDED | no dup; RC-03 landed init_enclave→init_within(NA_SRAM_CEILING_BYTES) at :137-145/:204-211 (also supersedes FIX-04's one-liner) |
| 3 | crates/cgap-ffi/src/stats.rs | 'v'→u64 typo (RC-11 W6.4) | SUPERSEDED | owner landed `pub quota_total_refusals: u64` :106 |
| 4 | crates/cgap-ffi/src/sovereign.rs | OsStr unblock (FIX-07 W5) | SUPERSEDED | owner FIX-01 landed canonical_key with stable OsStrExt :899-933; FIX-01 receipt cites the unblock |
| 5 | crates/cgap-git/tests/posh_refusal_matrix_court.rs | stored-framing clean_pack (FIX-05 W6.2) | CONSISTENT | :65-76 hand-built stored framing via encode_object_header+sha1; 4/4 pass |
| 6 | crates/cgap-git/src/ref_store.rs | pub(crate) implcell (RC-02 SLA section) | SUPERSEDED | `pub(crate) mod implcell` :161 with `use core::cell::Cell` :162 present (transient drop repaired); consistent with RC-01 critical seam |
| 7 | crates/cgap-git/tests/quota_alloc_purity_court.rs | extern crate alloc (FIX-04 §6.2) | SUPERSEDED | `extern crate alloc;` :27 landed by owner RC-14; compiles + passes |

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/r17-target cargo build -p cgap-p28 -p cgap-bn3 -p cgap-ffi` → exit 0 (Finished dev profile 1.18s)
- `CARGO_TARGET_DIR=/tmp/r17-target cargo test -p cgap-git --test quota_alloc_purity_court --test posh_refusal_matrix_court` → exit 0 (4 passed + 1 passed, 0 failed)

## Standing

ALIVE. Every verdict above re-read from disk this session and backed by the
exit-0 commands above. Zero RESIDUE-FOUND; zero edits needed, so no
disclose-in-both-directions entries from this lane.

## Fences

No shared files touched, so no regression surface from this lane. Fences
observed: cgap-p28/bn3/ffi compile green (RC-11/FIX-07 unblock surfaces),
both cgap-git courts green (FIX-05/RC-02/RC-14/FIX-04 surfaces).

## JSON terminus

```json
{"lane":"R-17","files_written":["docs/jira/v26.11.2/lanes/lane-r17-unblock-supersession.md"],"commands":["cargo build -p cgap-p28 -p cgap-bn3 -p cgap-ffi -> exit 0","cargo test -p cgap-git --test quota_alloc_purity_court --test posh_refusal_matrix_court -> exit 0"],"courts_run":["quota_alloc_purity_court","posh_refusal_matrix_court"],"standing":"ALIVE","divergences":[]}
```
