## 6. Self-audit

Executable checks run this session against the on-disk file (commands and
exits recorded here; rerun them, they are the court):

- A1 line length: `awk 'length > 100 {c++} END{print c+0}'` over this
  file. Convention exemption named in advance, per the v2 finalize
  precedent (receipt: ../../v26.11.1/waves/paper-finalize.md:34-36,
  divergence D2): markdown table rows are exempt; every other line must
  be <= 100 chars. Observed count and line classification in the receipt
  `../lanes/lane-paper-assembly.md`.
- A2 untagged-numeral sweep: paragraph-scoped pass over every
  digit-bearing line; each residual line must be (a) tag-bearing,
  (b) an identifier or path (file paths, frame-layout tokens like
  0x5749_5230, section names), or (c) a structural heading. Zero
  unlawfully untagged quantitative claims is the pass condition.
- A3 receipt existence: every `../...md:<line>` cited in this file must
  resolve to an on-disk file at audit time; a missing receipt file is a
  FAIL, never a silent pass (method: `test -f` per cited path).
- A4 parts identity: each file under `parts/` is a byte-identical
  subsequence of this file (assembly is mechanical concatenation; the
  check is `cmp` of each part against the corresponding byte span).
- Tag census at assembly time: receipt-tag count, [TARGET-v26.11.2]
  count, and [CITE-TBD] count recorded in the receipt
  `../lanes/lane-paper-assembly.md`; a re-read at use time supersedes
  any recalled count.

ERRATA policy: fix-forward on this file only; a defect in a landed
receipt number is corrected by a dated errata note citing the original
file+line, never by rewriting the receipt. Landed receipt numbers are
immutable inputs.

