# w1-13-registry-sweep

## 1. Scope

Lane W1-13 charter doc: `docs/jira/v26.11.2/08-registry-sweep.md`.
Sole writes: the doc + this receipt. No git commands. No writes to
/Users/sac/stardf.

## 2. Commands + exits

- `sed -n '1,60p' /Users/sac/cgap/Cargo.toml` — exit 0 (members list
  re-read from disk, never from memory: 22 member paths, cli excluded
  by an inline NOTE).
- `cargo metadata --no-deps --format-version 1 --manifest-path
  /Users/sac/cgap/Cargo.toml` piped to a python package count —
  printed 22, exit 0.
- `sed -n '80,140p' docs/jira/v26.11.2/waves/v2-07-census.md` — exit 0
  (live API states: 19 cgap-family PUBLISHED, 404s x4 integration/
  beam/court/cli, bench squatter 1.1.0).
- `sed -n '1,140p' docs/jira/v26.11.1/waves/coord-publish-record.md` —
  exit 0 (five-crate publish chain, one attempt each, dependency
  order, independent witnesses).
- Chunked writes of the doc (Write + 2 heredoc appends, exit 0 x3).

## 3. Courts run

None — the charter lease holds no court. Anti-vacuity is named in doc
§2.5/§5: the future `registry_sweep.py` self-check must flip a mutated
expectation or exit nonzero. This lane consumed no API calls; all
registry states cited are second-hand from the census receipts and are
marked as provenance, not current truth.

## 4. Standing

PARTIAL. The charter document is written and grounded (members
enumerated live; every cited state has a receipt pointer); the compiler
`scripts/registry_sweep.py` is DESIGNED, not implemented — that is a
[TARGET-v26.11.2] follow-up, and the doc says so.

Divergences:

1. First Write of the doc corrupted (duplicated rows, "ALived",
   "PUBLISHED-227.11.1"); rewritten in chunks per the corruption law.
2. "22-crate" is member-level: cargo metadata yields 22 packages
   including ontologies/fibo + cloud; cli is a 23rd standalone
   non-member row (charter §1 note).
3. cgap-bn3 / cgap-gpu / cgap-crypto per-crate cells are carried from
   family-level census verification (census:101); per-crate cells for
   rows 12-19 carry the same provenance caveat pending the compiler.

## JSON footer

```json
{
  "lane": "W1-13",
  "files_written": [
    "docs/jira/v26.11.2/08-registry-sweep.md",
    "docs/jira/v26.11.2/waves/w1-13-registry-sweep.md"
  ],
  "commands": [
    "sed -n '1,60p' /Users/sac/cgap/Cargo.toml (exit 0)",
    "cargo metadata --no-deps (22 packages, exit 0)",
    "sed -n '80,140p' waves/v2-07-census.md (exit 0)",
    "sed -n '1,140p' v26.11.1/waves/coord-publish-record.md (exit 0)",
    "chunked doc writes (exit 0 x3)"
  ],
  "courts_run": [],
  "standing": "PARTIAL",
  "divergences": [
    "first doc write corrupted; rewritten in chunks",
    "22-crate is member-level (fibo+cloud included, cli 23rd standalone row)",
    "rows 12-19 per-crate cells carried from family-level census:101 pending compiler"
  ]
}
```
