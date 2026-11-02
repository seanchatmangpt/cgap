# Lane IPC-6 — interop-chicago receipt sealer (honesty-gated emitter)

Standing: ALIVE (all commands below ran this session, exits shown).
Subject: `courts/interop-chicago/cmd/seal_receipt/main.go` (sole code file),
`courts/interop-chicago/cmd/seal_receipt/receipt-template.md` (template).

## Identity / Authority

Dispatch prompt LANE IPC-6. Owned files: the sealer dir above + this
receipt. No IPC-1 stub existed on disk (`courts/` did not exist), so the
sealer is created fresh — no stub replaced.

## Commands+exits

```
$ go version                                    # exit 0 — go1.23.4 darwin/arm64 (IPC-0 gate passes)
$ go vet main.go                                # exit 0
$ go build -o /tmp/ipc6-seal_receipt main.go    # exit 0
```

## Honesty gates (self-lint, fail-closed)

- SL-001: quantitative bullet without method disclosure
  (batch-amortized | machine-dependent | load-sensitive) — refused.
- SL-002: bullet citing no receipt path — refused.
- SL-003: bullet number with no this-session command exit in its input —
  refused. (Also fires when a bullet cites a CTQ name with no row.)
- Refusal = exit 1, named `REFUSED SL-00x` lines on stderr, output file
  NOT written. Mirrors the DOC-RECEIPT fail-closed spirit of
  `lanes/lane-gap-05-receipt-lint.md`.

## Courts table

| Court | Mutation (anti-vacuity) | Flip witness (command+exit) | Exit |
|---|---|---|---|
| honesty-gate flip | fixture with uncited quantitative claim (no method, no receipt, no exit) | `/tmp/ipc6-seal_receipt -in bad.json -out bad.md` -> 3 REFUSED SL-001/002/003 lines, `bad.md` not created (ls: No such file) | 1 |
| honesty-gate flip (fixed) | same fixture with method+receipt+exit filled | `/tmp/ipc6-seal_receipt -in good.json -out good.md` -> EMITTED 939 bytes, sha256 transcript root sealed | 0 |
| vacuity guard | the refusal itself is the flip; a green run without the bad-fixture refusal would be VACUOUS | both runs above, same session | 1 / 0 |

## Transcript root divergence (named)

sha256, not BLAKE3 — Go stdlib has no BLAKE3. Named in the emitted
receipt and in the template; not faked. Root is computed over the
receipt bytes with trailing whitespace trimmed, before the root line.

## Consequence / fences

New files only; zero edits to existing files. Fence check: this lane
touches no Rust crates, no scripts/gates, no shared lib.rs — nothing to
regress. `go vet` clean. Fixtures live in /tmp/ipc6-sandbox (sandbox
law); the shared tree holds only main.go + template + this receipt.

## Standing

ALIVE: build (exit 0), refused-flip (exit 1, SL lines named), emit
(exit 0) all witnessed this session against the exact subject.

## Divergences/disclosures

- No IPC-1 stub found to replace (dir absent); fresh creation recorded.
- BLAKE3 -> sha256 divergence, named above.
- Sandbox fixtures (`/tmp/ipc6-sandbox/{bad,good}.json`) are lane debris
  in /tmp, not tree debris; left in place as the flip replay corpus.
