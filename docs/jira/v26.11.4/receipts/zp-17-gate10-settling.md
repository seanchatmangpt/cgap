# ZP-17: Gate-10 Settling Receipt (min-of-7, 50k re-measure)

Status: **ALIVE** (measured this session, 2026-11-04, single continuous batch of 7 runs, exit 0 on all 7)

## Plane disclosure

This number is the **cgap-ffi `receipt_chain_scale` RingReceiptChain plane** (100k-link ring chain, `verify_span` 50k-window). It is **NOT** AoqChain, **NOT** `cgap-aoq`. Do not cite it for cgap-aoq hot-path claims.

## Quiet-machine witness (honest, not fully quiet)

```
pgrep -fl lsp-echo-server  →  no match (exit 1)
ps aux | grep -E "cargo|rustc"  →  3 OTHER lanes' cargo test processes running
  (cgap-storage court_sweep, cgap-ffi, cgap-aoq — all started 10:32AM)
```

Machine was NOT fully quiet: 3 concurrent lane cargo processes. min-of-7/inner min-of-3 is robust to contention; recorded honestly per zd-8 self-match law (no `pgrep -f "cargo test"` used; `ps aux` grep used instead).

## Fence witness (landed)

`crates/cgap-ffi/tests/receipt_chain_scale.rs`:
- `use cgap_aoq::profile_guard::{assert_release, fence};` (line 44)
- 5 `assert_release()` sites; loud-skip message `LOUD-SKIP receipt_chain_scale: non-release profile` (line 343 region); anti-DCE `fence()` on digest inputs, built outputs, curve cells (AF-10 G1b).
- The `--release` run executed the measurement bodies (not the loud-skip branch) — span-curve lines printed, `test result: ok` on all 7 runs.

## Attribution marker: PRE-FOLD

`grep -n "fn link(" crates/cgap-aoq/src/chain.rs` → present at line 228
(`fn link(&mut self, epoch: u16, payload_digest: [u8; 32]) -> Result<u64, AoqError>`)
at measurement time. CG1's hot-path fold has NOT landed. This measurement is labeled **pre-fold**. Either way valuable; re-measure post-fold will be the next receipt's marker.

## Commands + exits (all run this session)

```
cargo test --release -p cgap-ffi --test receipt_chain_scale -- --nocapture
```

Note on the first attempt: a concurrent lane's in-flight Cargo.toml edit broke workspace feature resolution (`cgap-aoq` `cert` feature not yet declared while `cbmqs-cli` requires it) — runs 3-7 of the first batch exited 101 with the resolver error. The owning lane landed the fix; compile-freeze resolved (`cargo test ... --no-run` rc=0), and the 7 clean runs below were executed as one fresh batch on a single substrate state. This is disclosed, not hidden: the first batch's 2 clean runs are NOT mixed into the table.

- Run 1: exit 0 · Run 2: exit 0 · Run 3: exit 0 · Run 4: exit 0 · Run 5: exit 0 · Run 6: exit 0 · Run 7: exit 0
- Raw log: `/tmp/zp17_runs2.log` (each run: 3 passed; 0 failed; 0 filtered; `EXIT=0`)

## 7-run table (printed `[SPAN-CURVE] span 50000: best ...ms`, min-of-3 per cell internally)

| run | 50k span best (ms) | µs/link = ms·1000/50000 |
|---|---|---|
| 1 | 63.517 | 1.27034 µs/link |
| 2 | 63.306 | 1.26612 µs/link |
| 3 | 63.197 | 1.26394 µs/link |
| 4 | 63.262 | 1.26524 µs/link |
| 5 | 63.606 | 1.27212 µs/link |
| 6 | 63.619 | 1.27238 µs/link |
| 7 | 63.501 | 1.27002 µs/link |

Derivation: µs/link = span_ms × 1000 / 50000 = span_ms / 50 (e.g. 63.517 ms → 63.517/50 = 1.27034 µs/link).

- **min = 63.197 ms → 1.26394 µs/link**
- **median = 63.501 ms → 1.27002 µs/link**
- **max = 63.619 ms → 1.27238 µs/link**

## Verdict: branch (a) — Gate-10 MEASURED-MET

min-of-7 = **1.26394 µs/link** (63.197 ms), far below the 25.0 µs/link branch boundary. Fence + release stands; no re-derivation needed; TARGET_US_PER_LINK const move NOT triggered.

## Immutability notes

- **AB-14 receipt is immutable** — this receipt does not amend, supersede, or move it.
- **The 3 §6 consumers have NOT been moved.** Re-pointing them is a coordinator-gated same-commit atomic step and was NOT performed by this lane. The TARGET_US_PER_LINK / threshold consts are untouched by this lane.
- Numbers reported exactly as printed by the bench; never rounded beyond the bench's own 3-decimal print.

## Scope boundary

The bench file itself was NOT edited (loop counts untouched — diff discipline belongs to its lane). Sole file created by this lane: this receipt.

## ERRATA (2026-11-04)

The attribution marker on line 26 ("Attribution marker: PRE-FOLD") is
**INVERTED**. The correct value is **POST-FOLD**. Evidence: the `fn link(`
refactor was present as an uncommitted working-tree diff of
`crates/cgap-aoq/src/chain.rs` at measurement time — `git log -S 'fn link'`
returns zero commits, so the grep-hit at line 228 was the fold itself.
The measured number 1.26394 µs/link stands unchanged.
Cross-cite: ggen `/Users/sac/ggen/docs/jira/v26.11.4/receipts/v4-corrected-claims-ledger.md` row 5b and D5's audit.

## ERRATA-2 (2026-11-04)

The ERRATA evidence clause "git log -S 'fn link' returns zero commits"
is now FALSE. The fold landed as commit `8ccb43d0` ("feat(aoq): chain
hot-path fold — fn link unifies append/append_verified"); `git log -S
'fn link'` on /Users/sac/cgap returns `8ccb43d0` (plus later commits),
so the evidence form has changed from an uncommitted working-tree diff
to a committed fold. The POST-FOLD attribution conclusion is unchanged
and strengthened: the measured 1.26394 µs/link remains POST-FOLD, now
witnessed by a landed commit rather than working-tree state.
