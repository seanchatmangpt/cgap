# RED-15: PII Redaction Leakage & Reconstruction Attack

Lane: RED-15 (read-only over crates/). Date 2026-11-02.
Target per dispatch: "SWAR bitwise PII stripping mask"
(cgap-branchless sieve / cgap-arrow SWAR kernel family).

## Verdict

**PARTIAL / UNSUPPORTED** — the named target has NO on-disk referent.
There is no PII-redaction, pseudonymization, or anonymization
implementation anywhere in `crates/`:

- `grep -rniE "redact|\bpii\b|pseudonym|anonymi" crates --include="*.rs"` -> exit 1 (zero matches)
- `grep -rniE "redact|pii|pseudonym|privacy|anonym" docs/jira/v26.11.2` -> exit 1 (zero matches)

The only redaction-adjacent claim in the repo's history is a
process claim, not a code claim: f3-05's "redaction sweep -> 0
matches" (`docs/jira/v26.11.1/waves/f3-05-auth.md:54`) — a receipt
secrecy sweep, not data-plane redaction. No data-plane redaction
claim exists to breach; therefore **zero BREACH-CONFIRMED verdicts**
(a breach requires a surviving claim; here none exists).

## What actually exists (referent correction)

The "SWAR mask" on disk is a **validity-masking checksum kernel**, not PII stripping:

- `crates/cgap-arrow/src/digest.rs:29-44` — `mask_words_hashed`:
  `out[i] = valid[i] & pred[i]` with a rotate-xor digest fold
  (`fold_word`, digest.rs:21-23). Self-described "NOT CRYPTOGRAPHIC
  ... will lose to an adversary" (digest.rs:3-12).
- `crates/cgap-branchless/src/swar.rs` — SWAR arithmetic kernels
  (popcount/permute class), no PII semantics.
- `crates/cgap-git/src/commit_synth.rs:19-24` — BLAKE3 tree digests,
  author/committer identity lines carried verbatim.

## Cryptanalysis over the surfaces that exist

No pseudonymization exists, so re-identification vectors are assessed
against the deterministic-hash surfaces:

### V1 — Unsalted BLAKE3 over low-entropy identifiers (cgap-git)

BREACH-CLASS note (no redaction claim to breach): commit_synth
addresses content with unsalted BLAKE3
(cgap-git/src/commit_synth.rs:19-24, 178). This is git-style content
addressing, deterministic by design; an email-shaped field hashed
here WOULD be dictionary-reversible (BLAKE3("x@y.com") is a fixed
constant any party can recompute). The code makes no redaction claim
(UNSUPPORTED, commit_synth.rs:9-10 carries author/committer lines
verbatim), so verdict is **UNSUPPORTED**, not BREACH-CONFIRMED.

### V2 — mask_words_hashed differential leakage across frames

**ALIVE-RESISTANT to re-identification, but a detectability hole:**
the digest folds only masked OUTPUT words (digest.rs:42: `out[i] =
valid[i] & pred[i]` pinned at digest.rs:13-15: flipping `valid` bits
beneath zero `pred` bits changes no digest bit). Corollary: two raw
inputs differing arbitrarily in masked-off lanes are digest-equal —
corruption in masked-off lanes is invisible. That is documented,
tested semantics (differential test flips bits "at several positions"),
not a defect against any claim. PII could sit in masked-off lanes
with digest-identical output — but since no redaction is claimed,
nothing is breached.

### V3 — commit_synth identity leakage

**UNSUPPORTED**: `author`/`committer` lines stored verbatim
(commit_synth.rs:9-10, 58-59, 182-183 `author_ts`/`committer_ts`),
timestamps in the clear. Any redaction layer would have to be
upstream of commit_synth; none exists.

## Fences

None — read-only lane; zero diffs to shared code; no gates could
regress. Commands were grep (exit 1 = zero matches, grounded above)
plus file reads.

## Commands (exit codes)

1. `grep -rniE "redact|\bpii\b|pseudonym|anonymi" crates --include="*.rs"` -> 1 (no matches)
2. `grep -rniE "redact|pii|pseudonym|privacy|anonym" docs/jira/v26.11.2` -> 1 (no matches)
3. `grep -rn "PII\|redaction" README.md docs/jira/v26.11.1` -> 0, one hit: f3-05-auth.md:54
4. reads: cgap-arrow/src/digest.rs, cgap-branchless/src/swar.rs,
   cgap-git/src/commit_synth.rs, f3-05-auth.md
