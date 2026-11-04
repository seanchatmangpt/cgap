# L7 Push-State Sweep — v26.11.4

Standing: ALIVE (T2 sweep 2026-11-04, ls-remote exit-code gated)

## Method

Three passes: baseline T0 (2026-11-04T20:04Z), T1 (~+15 min,
2026-11-04T20:31Z), final T2 (2026-11-04T21:04:33Z). Remote presence
verified by `git ls-remote origin | grep $(git rev-parse HEAD)` (exit-code
gate), never push output. Final table uses T2.

## Final table (T2, 2026-11-04T21:04:33Z)

| repo | HEAD | branch | dirty | ahead/behind | pushed |
|---|---|---|---|---|---|
| cgap | 23436067ab758dfeb034a2957406aa6906c56c63 | main | 25 | 0/0 | yes |
| arcus | 4dd6e39e645e9268c5fd1624b6c22855cd9ad44f | main | 9 | 0/0 | yes |
| ggen | 377442907ae2e5fb2c5d6bb2820a4a9f932cb9eb | v26-11-3-wave | 12 | 1/0 | **NO** |
| ggen-marketplace | 7e9a9853e59143603132af3cceef73a8ab2eb6e4 | main | 0 | 0/0 | yes |
| smalt | 382db62fdb9a0501697f262342622c76287bf665 | main | 259 | 1/0 | **NO** |
| lsp-max | 1f6f9bcbbeed99c6db33a2c55040a389965f3867 | master | 0 | 0/0 | yes |

## Flags

- **ggen HEAD NOT on origin** (377442907ae2e5fb2c5d6bb2820a4a9f932cb9eb,
  branch v26-11-3-wave, 1 ahead). Unpushed at final sweep.
- **smalt HEAD NOT on origin** (382db62fdb9a0501697f262342622c76287bf665,
  1 ahead, 259 dirty — dirty count jumped 1 -> 259 between T1 and T2,
  likely mid-flight lane writes or a build dir; investigate before any
  fleet-wide `git add -A`-class operation).
- **arcus**: remote existed as PENDING-REMOTE at T0 (no remote configured)
  and the creation lane landed mid-sweep; by T2 origin is configured and
  HEAD 4dd6e39e is present on it. Resolved — no action needed.
- All six HEADs moved at least once across T0/T1/T2 — lanes were landing
  continuously during the sweep window.

## Baseline (T0) for drift comparison

| repo | HEAD | dirty | ahead/behind | remote-present |
|---|---|---|---|---|
| cgap | b5c6d5d0fc1ca96ed64a4ea53a0a09dbd30d64b5 | 32 | 8/0 | NO |
| arcus | 691c27727742cc3412c5cab86ad75f0667750828 | 10 | no upstream | no remote |
| ggen | 15bd665677b063aeebca7c5e299867ed5ed26f74 | 29 | 0/0 | NO |
| ggen-marketplace | ee4c589be6aa7ce0cd40e105a34db53605f60e9c | 3 | 0/0 | yes |
| smalt | d689e03920089a4efad19e32041c80034a08ea65 | 5 | 0/0 | yes |
| lsp-max | e41e957596c4aa85cb5e72e3cd4fc4157f408f7c | 2 | 0/0 | NO |

## Remotes

- cgap: origin https://github.com/seanchatmangpt/cgap.git
- arcus: origin https://github.com/seanchatmangpt/arcus.git (added mid-sweep)
- ggen: origin https://github.com/seanchatmangpt/ggen.git
  (+ upstream rust-starter/rust-starter.git)
- ggen-marketplace: origin https://github.com/seanchatmangpt/ggen-marketplace
- smalt: origin https://github.com/seanchatmangpt/smalt.git
- lsp-max: origin https://github.com/seanchatmangpt/lsp-max
