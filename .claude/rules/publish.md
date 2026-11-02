---
paths:
  - "**/Cargo.toml"
---

# Publish law (crates.io)

- One publish attempt per crate per version — never re-fire a failed
  attempt without a named cause. On 429, honor Retry-After and
  **stop-on-slide** (do not advance crates while one is unresolved).
- House style: one-line capability `description` (no marketing),
  workspace-relative `license-file = ".../LICENSE.md"`, explicit
  `publish = true/false` on every crate.
- `publish = false` until a receipt-backed court run exists for the
  crate under `docs/jira/*/waves/`; flipping to `true` is a receipted
  transition, not a default.
- Umbrella version pins move as one unit (siblings = root version).
- **Publishes FROZEN** (`waves/p4-terminus.md`, `waves/publish-halt.md`,
  2026-11-01); no attempt until the operator lifts the freeze. Lift
  sequence: `waves/publish-halt.md:31-42` — cgap-gate first publish;
  repair republishes owed on lift: cgap-blake3 26.11.3 (NEON opt>=1
  divergence), cgap-arrow 26.11.3 (stale embedded generated_version
  26.10.29), cgap-parquet 26.11.3 (RingSink), cgap-bn3 republish
  (no_std, W6-BN3NS); cgap-integration first publish (never on the
  registry). Header parity: CLOSED by F8, no longer a lift blocker.
- **Frozen guard hook**: `.claude/hooks/publish-guard.sh` enforced (C4);
  publish attempts while frozen are refused mechanically.

Witnessed: `docs/jira/v26.11.1/waves/f3-publish-ledger.md`
(per-crate dry-run/publish/API-verify table, zero observer disagreement).
