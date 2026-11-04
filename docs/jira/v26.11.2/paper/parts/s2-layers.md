## 2. Layer discipline (fence F1 and the token fence)

Two fences bind every number in this paper. Violating either falsifies it
(§5).

F1 — admission kernel vs wire protocol. The 10.20 ns/frame fastest /
10.30 ns/frame median admission-kernel figures (quiet-host ingress, divan;
registered contention divergence 124.06 ns/frame slowest) live on a
DIFFERENT layer than every µs/round-trip and B/envelope figure here
(receipt: ../paper/position-paper-v3.md:207; underlying receipts
../waves/w3-l1-ingress.md:63,69-70 and ../11-measured-claims.md rows 5/28).
ns/frame and µs/round-trip are different instruments on different
workloads: never summed, never ratioed, never placed in one table without
the boundary label (receipt:
../bench-validate/p02-http-baseline.md:11-22). The ~700x gap between
10.20 ns/frame and the 7.40 µs stdio median is a layer gap, not a win
(receipt: ../bench-validate/p01-stdio-baseline.md:168-170).

Token fence — bytes are not tokens. Byte savings and token savings are
different layers of the same wire. The p17 leg measured BOTH with a real
tokenizer (tiktoken 0.12.0, o200k_base — not a bytes/4 heuristic) and
found the binary frame 14.06% smaller in bytes yet 27.90% LARGER in tokens
(receipt: ../bench-validate/p17-token-churn.md:106-110,112-120). This
paper therefore scopes "zero-copy" to BYTES, never to tokens. Any claim,
in this file or derived from it, that binary frames save agent tokens is
a falsifier of this paper (§5, F7).
