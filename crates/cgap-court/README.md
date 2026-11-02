# cgap-court

**Status: NOT-PUBLISHED** — verified against crates.io this session (no
crate record). Workspace-local court harness.

## What it is — the honest description

`cgap-court` is a **[[test]]-only harness package: it has no `src/` by
design.** The package is a `[lib]`-named shell whose entire body is one
integration test target, `court_verdict`
(`path = "../../tests/court_verdict.rs"`), declared in Cargo.toml. It
exists so `cargo test -p cgap-court` runs the whole-repo court verdict
harness under one command; it is not a library, publishes nothing, and
any README implying a public API surface would misrepresent it. This
document is the full description.

## Test receipt (run this session)

```
cargo test -p cgap-court   # exit 0
  14 passed; 0 failed  (court_verdict target)
```

Includes named courts such as `a14_dam_ingress_court` (gate lane A14).
Mutation ledger: `a20-kill-ledger.json` sits beside Cargo.toml — the
A20 kill ledger this harness maintains.
