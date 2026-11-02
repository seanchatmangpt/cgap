# repair-14 — scripts pyright ✘ cleanup (W11 fix list)

Lane F15, 2026-11-02, repo /Users/sac/cgap (main). Owner files:
`scripts/verify_disassembly_invariants.py`,
`scripts/shacl_negative_corpus.py`, `scripts/shacl_negative_corpus_b.py`,
this receipt. Typing/narrowing only — no behavioral changes intended or
observed.

## ✘ counts (pyright 1.1.409, this session)

| file | before | after |
|---|---|---|
| verify_disassembly_invariants.py | 6 | 0 |
| shacl_negative_corpus.py | 25 | 0 |
| shacl_negative_corpus_b.py | 12 | 0 |

Combined run: `pyright` on all three → `0 errors, 0 warnings` (exit 0).

## Fixes per file

- verify_disassembly_invariants.py (m.group(None) at :145/:325/:326):
  minimal local narrowing — `assert im.lastindex is not None` before
  `im.group(im.lastindex)` (:145); :325/:326 folded into
  `im.lastindex is not None` guard on the existing `if im and ...`.
- shacl_negative_corpus.py: `read_family(g, shape: URIRef)` and
  `build_graph(..., node: URIRef, ...)` (call sites pass URIRef, not
  Namespace); `assert shape_class is not None` /
  `assert path is not None` (value() narrowing);
  `g.value(..., default="")` → `g.value(...)` + `or ""` (identical
  None/""-coalescing semantics); `assert isinstance(results_graph, Graph)`
  after `pyshacl.validate`.
- shacl_negative_corpus_b.py: `internal_hash()` digest branch rewritten
  as `isinstance(ih, int)` (same int→str().encode / else .digest()
  split the hasattr form expressed); `assert isinstance(results_graph,
  Graph)` after pyshacl.validate; `row.dutyAction` → `getattr(row,
  "dutyAction", None)` with URIRef coercion (URIRef subclasses str, so
  the isinstance branch is identity for the URIRef case); return type
  widened to `URIRef | None` kept, satisfied via the coercion.

## Verification (this session, exact subjects)

- pyright all three → 0 errors (exit 0).
- `python3 -m py_compile` all three → exit 0.
- Spot-run A (shacl_negative_corpus.py): before-copy (HEAD via
  `git show`, ROOT pinned to the real repo since the script resolves
  ROOT from `__file__`) vs after-copy run from /tmp with pinned ROOT:
  before exit 0, after exit 0, **byte-identical stdout** (7/7 FLIPPED,
  self-check PASS, N-07→REFUSAL-09 divergence note preserved).
- Spot-run B (shacl_negative_corpus_b.py): same method (REPO pinned):
  before exit 0, after exit 0, **byte-identical stdout** (VERDICT PASS,
  13/13 iso-exact restores, PC-1 DETECTED).
- Spot-run C (verify_disassembly_invariants.py): full court run in the
  shared tree, exit 0, "AE4 disassembly-invariants court: CLEAN".
  No HEAD copy exists (file is untracked/new), so the before-run is the
  pre-edit state of this session; the fix sites are only exercised on
  the mutation path, and the mutant + self-check lines still PASS.
- No git writes; shared tree touched only in the three owned scripts
  plus this receipt.
