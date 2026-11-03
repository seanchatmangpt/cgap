# Lane PUB-02b — release-gates flag table refresh (2026-11-02)

W1 subject: docs/jira/v26.11.2/01-release-gates.md (sole owned file).
W2 diff: appended dated section "G1-PUBLISH flag table — refreshed
re-derivation (2026-11-02)" after the session-evidence block;
prior "19/22" text at :27/:32 retained verbatim with ERRATA pointer
(superseded by 23 members / 20 true / 3 false; drift sources:
ontologies/ members fibo+cloud, branchless DOD-02 flip; CLAUDE.md
"12/23" also superseded). Merged, never clobbered.
W3 commands/exits:
- find audit/pub-02... (receipt located at
  docs/jira/v26.11.2/audit/pub-02-flag-table-rederivation.md) exit 0
- grep -n "^publish" crates/{integration,wasm-footprint}/Cargo.toml
  exit 0 — both `publish = true`; exposed PUB-02 W4 annotation typo
  ("moot: publish=false" on cgap-integration vs its own W2 row 18
  flag=true). Table follows the disk-verified flag (404-pending,
  registry-absent), disclosed here.
- grep -c '^| ' / wc -l on the target: 31 table rows, 380 lines on disk.
W4 verification: 23 member rows + 6 non-member rows = 29 data rows
(31 incl. headers), all cells cross-checked against PUB-02 W2/W3/W4;
no >100-char non-table lines; one self-caught cell error fixed
(cgap-beam registry: receipt says 404, initially wrote "exists").
W5 figures NOT re-invented: 20 true / 3 false, 19/20 exists,
bench collision 1.1.0, branchless 404-pending, wasm-footprint
UNVERIFIED-registry — all copied from the receipt verbatim.
W6 fences: root CLAUDE.md Publish-law paragraph still carries
"12/23 explicit, 9 true / 3 false" — NOT owned by this lane; left
untouched, flagged for owner lane. No git commands; no shared-tree
mutation outside the owned file.
W7 standing: ALIVE (table landed + byte-verified on disk this
session). Divergences: PUB-02 W4 cgap-integration annotation typo
(disk wins: publish=true, registry-absent).
