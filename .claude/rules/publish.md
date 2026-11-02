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

Witnessed: `docs/jira/v26.11.1/waves/f3-publish-ledger.md`
(per-crate dry-run/publish/API-verify table, zero observer disagreement).
