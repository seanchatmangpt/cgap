# ERRC-A — Marketplace packs A-D innovation evaluation

Lane: ERRC-A (court-auditor, read-only on /Users/sac/ggen-marketplace). Date: 2026-11-01.
Subject: /Users/sac/ggen-marketplace/packs/[a-d]* — 107 packs. cgap repo: receipt file only.

## Commands + exits (auditor-run)
- `ls packs/` -> 0 (268 dirs total)
- `python3 scripts/qualify_packs.py --report /tmp/...` -> 2 REFUSED:MANIFEST_MISSING:smalt-core-pack (fail-closed, repo-wide court BLOCKED)
- `python3 scripts/marketplace.py validate` -> 2 (same refusals); `catalog` x2 -> 2 each, outputs 0 bytes; `cmp` identical only vacuously (both empty)
- `qualification/baseline.json` (ggen 26.9.28): counts ALIVE 289 / REFUSED 8 / SKIPPED 1 / WARN 9; a-d slice: 107 packs = 97 ALIVE, 6 REFUSED, 4 WARN
- `git log -1` recency sweep: a-d pack mtimes 2026-10-10 through 2026-10-31 (cgap-pack 10-31, ash-r2rml*/athena-matter 10-17, affidavit-pack 10-13, bulk 10-10)

## Non-ALIVE a-d packs (from baseline)
- REFUSED: aaif-profile-tailoring-pack, aaif-vanilla-pack, ash-pplan-chaos-pack, ash-pplan-igniter-pack, ash-pplan-protocol-court-pack, ash-pplan-store-conformance-pack
- WARN: automatic-autonomic-operations-pack, certification-assist-evidence-control-pack, certification-assist-pack, clap-noun-verb-crate-pack

## ERRC classification (a-d slice)
- ELIMINATE: affidavit-pack (self-declared DEPRECATED, superseded by affidavit-consumer-pack); clap-noun-verb-pack (DEPRECATED, superseded by schema/routing/crate split); chatman-ecosystem-v26-9-1-release-gate (self-deprecated as reusable authority); cs2-canonical-pack vs cs2-conformance-pack vs cs2-exact-subject-pack (three overlapping RFC-CS2-001 projections -> collapse into one canonical + one conformance consumer)
- REDUCE (merge partners named): ash-pplan-{chaos,protocol-court,store-conformance}-pack all generalize the same ash_pplan machinery and ALL REFUSED -> merge into one ash-pplan-court-pack; ash-r2rml-paas-pack + ash-r2rml-reactor-paas-pack (v26.8.26/27 siblings) -> single R2RML PaaS pack; autofde-{openclaw-bridge,sota-factory}-cnv-pack are two cnv: shards of one autofde CLI surface -> one cnv pack family manifest; certification-assist-pack + its evidence-control extension (both WARN) -> one pack, two profiles
- RAISE: the 6 REFUSED packs (fix admission before they rot further); 4 WARN packs; a2a-conformance/durability/hex-migration trio (v0.1.0, ash_a2a-pinned, no fresh conformance receipt since baseline ggen 26.9.28 while ash_a2a moved)
- CREATE-gap: cgap campaign needs a benchmark-court pack (Divan bench membrane + PARETO receipt as reusable capital) — no a-d pack serves it; also no pack packages the "guard-flip + metadata publish law" — court-kit-pack (untracked, no git history) is the closest and is itself unqualified

## cgap consumption candidates (a-d slice)
- chicago-tdd-tools-pack (v0.5.0, ALIVE) — CLI boundary courts for crates/cli
- court-kit-pack (ALIVE baseline; untracked files) — anti-vacuity refusal-register courts for cgap-gate
- chicago-graphlaw-court-pack, chicago-xaas-surface-pack — court-renderer patterns for cgap-court
- cargo-cicd-pack (v0.1.0, ALIVE) — typed Rust command catalog regen for crates/cli
- cgap-pack (v26.10.29, ALIVE, witnesses/ + gate-court.toml) — already cgap's own; keep qualified
- adapter-family-matrix (v26.9.30) — N-subject x N-consumer manufacture for cgap FFI membrane parity

## Top-3 ERRC innovations from this slice
1. ELIMINATE-then-qualify loop: the repo-wide qualification court is hard-BLOCKED on smalt-core-pack (manifest removed post-baseline) — one ELIMINATE (delete or re-manifest smalt-core-pack) unblocks the entire 307-pack conformance surface; highest leverage per diff in the marketplace.
2. Merge the REFUSED ash-pplan court trio into one pack and re-qualify — kills 3 REFUSED rows and removes O(N) duplicated generalization of the same machinery.
3. Promote court-kit-pack from untracked to qualified (pack.toml exists, no git history = no conformance trail) — it is the anti-vacuity substrate cgap's courts already follow; an unqualified court kit is a standing hole.

## Standing
- Marketplace a-d inventory: PARTIAL_ALIVE (baseline-qualified at ggen 26.9.28; live qualification court BLOCKED repo-wide by smalt-core-pack, pre-existing, not an a-d defect).
- This receipt: ALIVE (all commands reproduced this session; marketplace untouched; no git writes).

## Divergences
- qualify_packs.py and marketplace.py both exit 2 on smalt-core-pack REFUSED:MANIFEST_MISSING + REFUSED:ONTOLOGY_SOURCE_MISSING (pre-existing; smalt-core-pack holds ontologies/shapes/templates/vendor but no pack.toml).
- The catalog determinism "pass" on two empty outputs is vacuous — must not be cited as determinism evidence while the court is blocked.
- court-kit-pack files are untracked in git (no commit history): provenance hole for the pack the court doctrine points at.
