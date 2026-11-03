# lane-dod-10-claims

Implements PUB-10's rewording column
(`docs/jira/v26.11.2/audit/pub-10-claims-preflight.md`). 2026-11-02,
repo /Users/sac/cgap, branch main, NO git commands run.

## 1. Identity

- Lane DOD-10. Subject: 3 edited files + this receipt, all under
  /Users/sac/cgap, branch main.

## 2. Authority

- Dispatch prompt (DOD-10): rewordings only, files enumerated in the
  prompt; read `audit/pub-10-claims-preflight.md` fresh as spec.
- No sibling lane shares these paths.

## 3. Commands + exits

| Command (cwd /Users/sac/cgap) | Exit |
|---|---|
| grep sweep `5\.70\|5\.79\|175\.5M` in compendium | 0 (only errata lines 469-481 remain — correct: errata text must keep citing the refuted figure) |
| grep sweep `8-way AVX2` in cgap-blake3/Cargo.toml | 0 (only the qualified mention on :5) |
| grep sweep `9.6x\|102.7` in crates/bench/README.md | 0 (102.7 row + fresh 9.6x qualifier at :31-33) |
| grep count `7\.71\|7\.35` in compendium | 9 lines (rows + errata 2/3) |
| `cat >> …compendium` (Errata 3 append) | 0 |

All grep exits 0 above mean "match found" — i.e. the sweep verified
the expected residual state, not a vacuous no-match.

## 4. Consequence (per-file diffs)

- `crates/cgap-blake3/Cargo.toml:5` — description now reads
  "…4-way NEON parent Merkle reduction (8-way AVX2 path
  feature-gated; 26.11.3 re-witness owed)." per
  `audit/dod-cgap-blake3.md:91` (AVX2 opt>=1 divergence D1; NEON
  witnessed, AVX2 not re-witnessed since 26.11.3 republish is owed).
- `crates/bench/README.md:31-33` — 102.7 ms row now carries the
  freshness qualifier: as-of 2026-11-01 receipt; receipted 10.1x,
  fresh 9.6x (`audit/tps-01-reality-ledger.md:24`) — drifted.
- `docs/REVENUE_GENERATION_COMPENDIUM.md`:
  - :59 mermaid node → "7.71 ns/frame receipted Admission".
  - :237 crate table row → "7.71 ns/frame receipted (7.35-8.13
    re-derived; Errata 2)"; 175.5M derivative struck.
  - :288 master measurement row → fastest "7.71 receipted (1M) /
    7.35 fresh (10M)", median 8.13 ns, derivative 136.1M
    frames/sec/core (derived from 7.35 fresh: 1/7.35 ns), tagged
    "Errata 2".
  - Appended Errata 3 (:477-484 approx): records that the body rows
    are now corrected; supersedes Errata 2's "original lines
    retained unedited" clause. Errata 2 remains the re-derivation
    authority; this is an ERRATA-APPEND, not a rewrite of Errata 2.

Fences that could regress: none run — this lane edits prose/manifest
descriptions only, no code, no benches, no gates. The publish-guard
freeze hook is untouched; description change affects the
not-yet-published 26.11.3 republish surface only.

## 5. Courts

None applicable — no executable surface. [SKIP] no code path or gate
exists for manifest-description and prose rewordings; the named
anti-vacuity channel here is the grep sweep in §3, which flips
(holds real residual-state matches, not a vacuous zero).

## 6. Standing

ALIVE for the prose corrections (grep sweeps + on-disk reads this
session, exact subjects). UNVERIFIED (correctly out of lane scope):
whether the 26.11.3 cgap-blake3 republish actually re-witnesses the
AVX2 path; whether 136.1M frames/sec/core re-measures cleanly — it
is DERIVED from the receipted 7.35 fresh figure, not itself measured.

## 7. Divergences / disclosures

- Concurrent edit disclosed: while this lane held
  `crates/cgap-blake3/Cargo.toml`, another lane added
  `categories`/`keywords`/`[package.metadata.docs.rs]` (:9-10,:19-21).
  No conflict; both edits present on disk. This is the
  disclose-in-both-directions note for that overlap; the owning
  publish lane's receipt should cite this one.
- Compendium table rows exceed the 100-char line law — pre-existing
  house style of that file (one row = one line); kept.
- PUB-10 recommended "rewrite … or strike"; chose rewrite per
  dispatch. Pub-10 also flags cgap-court/cli/csct missing
  descriptions and a cgap-gate README qualifier — NOT in this lane's
  file list; left for PUB-01/PUB-02 owners.
