# AF-4: parse_meta read-ahead minimum (closes AB-9)

- **Lane**: AF-4 (implementation, src owner), 2026-11-03
- **Subject**: `crates/cgap-arrow/src/ipc.rs` — `parse_meta` truncation gate only
- **Finding** (AB-9, `ab-9-ipc-stress.md`): the parse seam demanded
  `buf.len() >= body_at + META_HEADER_SIZE` — 64 readable bytes past body
  start — although the gate's real consumption is `buf[8..72]` (the header)
  plus, on the AOQ1 branch, `buf[72..body_at]` (span padding), all strictly
  inside `body_at`. Emit-accepted tiny windows (body+pad < 56) classified
  `Truncated` on `read`: a deterministic, typed emit/read round-trip
  asymmetry.
- **Fix**: demand lowered to the true minimum, `buf.len() >= body_at`
  (`needed: body_at`). One surgical region; comment names AB-9/AF-4 and
  the law. AB-8's concurrent AOQ1 magic-fork edits in the same file are
  disjoint and untouched.
- **Classes intact**: `Truncated` still fires for windows shorter than
  prefix+span; shorter-body windows classify downstream `BadBodyLen`,
  missing-tail windows `MissingEos` (unchanged code in `read_with_meta`).
  C11's classification-identity sweep (read vs read_verified over 72
  single-byte mutations + every truncation cut) stays green.

## Per-cell table (AB-9 t4 sweep, body sizes 0..4097, both alignments)

Truncation condition was `window_len >= body_at + 64`, i.e. `body + pad < 56`:

| cell | before (AB-9) | after (AF-4) |
|---|---|---|
| align-8, body 0-48 B (`body+pad < 56`; sweep cells 0,1,7,8,9,15,16,31,32) | `Truncated` (emit-accepted, read-refused) | `read` green, byte-exact body |
| align-8, body >= 49 B | green | green (unchanged) |
| align-64, body 0 B | `Truncated` (emit-accepted, read-refused) | `read` green |
| align-64, body >= 1 B | green | green (unchanged) |

`read_verified` per-cell classification unchanged where parse was green:
align-8 `BodyDigestNoRoom { got: 8 }`, plain-emit align-64
`BodyDigestMissing`, verified emit align-64 green — now asserted at EVERY
cell (previously skipped on the parse-minimum `Truncated` cells).

## Exact-Truncated-needed updates (court (c))

| site | before | after |
|---|---|---|
| `ipc_courts.rs` C12 (`Truncated { needed, got }` pin, cut=100 in align-64 span) | `needed: 192` (`body_at+64`) | `needed: 128` (`body_at`) |
| `parse_meta` gate | `needed: body_at + META_HEADER_SIZE` | `needed: body_at` |

## Courts

- (a) Symmetry flip: `ipc_stress.rs` t4 rewritten — every emit-accepted
  cell must be read-accepted and byte-exact (was: recorded the asymmetry
  cells as an "observation, recorded not hidden"). Green.
- (b) C1-C13 canary: all 17 `ipc_courts` tests green both runs.
- (c) Exact-needed pins updated (C12), green both runs.

## Gate (commands + exits)

`cargo test -p cgap-arrow` — full suite, TWO runs against the final tree:

- Run 1: all targets 0 failed — 145 lib, 17 ipc_courts, 4 ipc_stress
  (t4 49.85s incl. the symmetric sweep), 5 ipc_body_integrity, 7+3+3+6+4
  stream_writer/property/validation/slab/other, mutation_kills executes.
- Run 2: identical, all 0 failed (ipc_stress 105.73s run incl. bench arm).
- Raw logs: two full-suite runs exit 0; zero `FAILED`/`error[` lines.

## Standing

ALIVE — ran this session against the exact subject
(`/Users/sac/cgap/crates/cgap-arrow/src/ipc.rs`, parse_meta gate region +
`tests/ipc_courts.rs` C12 + `tests/ipc_stress.rs` t4). Never run git.
