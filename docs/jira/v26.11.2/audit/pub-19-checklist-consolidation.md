# PUB-19 — Pre-publish checklist consolidation (one screen)

Date: 2026-11-02. Lane PUB-19, runs LAST of the first wave.
Deliverable is this file only. It consolidates PUB-01..18 into the
single ordered gate list the operator runs at freeze-lift. No code,
no manifests, no gate runs are made by PUB-19 itself.

## 0. Consumption ledger of PUB-01..18 (read at consolidation time)

Landed on disk at audit/pub-*.md this session:
- pub-08-version-unification.md (LANDED) — supplies §2 (pin drift
  census), §4 step-2 ship order, and the §4 step-3 falsifier grep.
PENDING (not on disk at write time; consume on landing):
- pub-01, pub-02, pub-03, pub-04, pub-05, pub-06, pub-07, pub-09,
  pub-10, pub-11, pub-12, pub-13, pub-14, pub-15 (first wave
  peers; may have landed after this write — re-check on use),
  pub-16, pub-17, pub-18 (in-flight).
Rule: each PENDING lane contributes (a) any DEFECT-OPEN rows it
files, and (b) gate-specific checks it owns below. A PENDING row
here is a placeholder, not a waiver — see §3 go/no-go.

## 1. Ordered gates (run in this order, stop at first non-zero)

| # | Gate | Command | Exit | Owner |
|---|------|---------|------|-------|
| 1 | manifest | grep -rn '26\.11\.[012]"' --include=Cargo.toml . | 1 (no hits) | PUB-08 |
| 2 | package dry-run | cargo package --dry-run --allow-dirty -p <crate> | 0 | PUB-19 |
| 3 | court tollgate | just courts | 0 | PUB-19 |
| 4 | semver | cargo update -w && lock grep 26.11.3 | 0 | PUB-08 |
| 5 | README | for f in crates/*/README.md: test -s || exit 9 | 0 | PUB-19 |
| 6 | docs.rs | cargo doc --no-deps --all-features -p <crate> | 0 | PUB-19 |
| 7 | secrets | tarball grep: no token/key/secret strings | 0 | PUB-19 |
| 8 | sequence | order per pub-08 §4 step 2 (bottom-up) | 0 | PUB-08 |

Exact forms PUB-19 will run (expand of the table above):

1. Manifest: `grep -rn '26\.11\.[012]"' --include=Cargo.toml .`
   Expected: exit 1 with ZERO output lines (the pub-08 §4 step-3
   falsifier; any hit is a stale pin = stop).
2. Package dry-run, per crate in ship order:
   `cargo package --dry-run --allow-dirty -p <crate>`
   Expected exit 0. Cargo.lock regen FIRST via
   `cargo update -w` (lock is regenerated, never hand-edited).
3. Court tollgate: `just courts` — expected exit 0, all three
   suites; fail-fast means any suite failure stops the gate.
4. Semver/lock: `cargo update -w`, then confirm every cgap
   package entry in Cargo.lock shows the lift version; csct 0.1.0
   is out of calendar and excluded.
5. README: every publishable crate ships a non-empty README.md
   beside its Cargo.toml. Loop:
   for d in <publishable crates>; do test -s "$d/README.md" || exit 9; done
   Expected exit 0; a missing/empty README is exit 9 = stop.
6. docs.rs: `cargo doc --no-deps --all-features -p <crate>`,
   expected exit 0 with zero warnings-as-errors; matches the
   docs.rs build config the registry will use.
7. Secrets: inspect each packaged tarball under
   target/package/*.crate before any publish actuator runs:
   grep for token/key/secret/password patterns in the .crate
   contents. Expected: zero hits. One hit = stop, file a defect.
8. Sequence: publish order is pub-08 §4 step-2 bottom-up order
   (blake3, arrow -> bont, crypto, git, gate, bosym -> core,
   p28, bn3, parquet -> storage, gpu, audit, integration ->
   ffi (only after parquet lift), umbrella, wasm-footprint
   (blocked, defect owed), cloud, fibo). The sequence gate is a
   paper gate: confirm the operator's publish queue matches the
   order before the first `publish` command is authorized.

## 2. Freeze context (binding, not advisory)

- Standing order: FROZEN-BY-OPERATOR
  (waves/publish-halt.md, 2026-11-02). NO publish actuator runs
  until the operator lifts the freeze in the publish-guard hook.
- Already-live (immutable, recorded): blake3, arrow, git,
  parquet, umbrella, core, storage, bont — all at 26.11.2.
- Blocked repairs owed ON LIFT: blake3 26.11.3 (D1 NEON),
  arrow 26.11.3 (D2 stale generated_version), parquet 26.11.3
  (RingSink). ffi blocked on parquet lift; wasm-footprint has
  the host-link defect (new class, diagnosis owed).
- Gate 2 (dry-run) is allowed under the freeze; `publish`
  itself is not. Dry-runs are read-only to the registry.

## 3. Go/no-go (single criterion)

GO iff every DEFECT-OPEN item filed by PUB-01..18 is either:
(a) FIXED with a command+exit witness run this session on the
exact subject (the register's CLOSED law), or
(b) covered by an operator exception receipt (dated, signed,
naming the defect and the lift it is waived to).
Anything else — including any PENDING pub-01..18 lane that
lands with an unresolved DEFECT-OPEN row — is NO-GO.

Falsifier (verbatim, do not reword):
the publish readiness Verify passes iff zero crates fail cargo
package, every pub module has a court+receipt, and the freeze
lift is operator-signed.

## 4. Stop rules

- Any gate non-zero exit: stop, no further gates run.
- Gate 1 hit, gate 5 exit 9, or any secrets hit: stop AND file
  a DEFECT-OPEN row before any re-attempt.
- All eight gates exit 0/1-as-expected plus a signed freeze
  lift receipt = GO; the operator's queue then runs in §1.8
  order only.
