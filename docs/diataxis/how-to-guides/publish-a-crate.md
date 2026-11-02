# Publish a crate

One dry-run, one real upload, one independent registry verification. Witnessed
for all five kernel crates in `waves/coord-publish-record.md`; the API-verify
verdict table is `waves/p3-api-verify-all.md`.

## Dry-run first — the resolution gate

```bash
cargo publish -p <crate> --dry-run
```

Exit 0 proves the manifest resolves against the live sparse index: the
witnessed chain resolved `blake3 ^26.11.1` from the registry only after the
predecessor's upload propagated (~15s wait per hop). Publish in dependency
order: cgap-blake3, cgap-arrow, cgap-git, cgap-parquet, cgap (umbrella).

## ONE real publish

```bash
cargo publish -p <crate>
```

Exactly one invocation per crate per version — the one-attempt law. A
failed attempt is not retried blindly; diagnose, fix, and treat the next
attempt as a new decision. The witnessed record shows 5 crates, 5 uploads,
exit 0 each, zero double uploads.

Disclose `--allow-dirty` if the tree is dirty (campaign receipts make it
dirty by design); never print the token.

## Verify on the registry API — independently

```bash
curl -s -A cgap-publish-receipt \
  https://crates.io/api/v1/crates/<crate> | python3 -m json.tool
```

Check: `max_version == <version>`, `num_versions == 2` exact for the
originals (26.10.29 + 26.11.1 — exactness catches double uploads), `yanked
false`, `published_by seanchatmangpt`. Also probe the sparse index for both
versions. Keep the raw JSON in `/tmp` as the receipt.

## Negative control

Probe a name that cannot exist (witnessed: `p313-definitely-not-a-real-crate`):
sparse index 404, API 400. If the probe returns 200, your check is not
checking anything.

## Authority boundary

Subagent publishes are REFUSED by the permission system (witnessed: the
f3-09..f3-13 markers). Publishes run coordinator-side; do not route them
through a lane.
