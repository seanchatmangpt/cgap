# tc-13-asha2a-feasibility

TC-13: the minimal first cgap <-> ash_a2a edge. Design brief:
cgapprd-08b W4. All referents file:line-grounded against the real
trees, 2026-11-02.

## W0 Headline

One NIF export, one digest decision, one differential court. The
edge is feasible WITHOUT a translation NIF and WITHOUT authority
work: cgap-beam gains `cgap_ingress_digest/1`, ash_a2a's existing
sha256-hex grammar is adopted at the membrane verbatim, and an
rf5-shaped oracle proves BEAM-side and cgap-side digests agree on
a fixed corpus. Rust surface: one function + one table row.
Elixir surface: one stub + one wrapper.

## W1 Component A — the admission-shaped NIF export

Pattern being extended: raw erl_nif ABI, zero external crates
(cgap-beam/src/lib.rs:3-13); entry table NIF_FUNCS: [ErlNifFunc; 21]
(lib.rs:480), CPU-heavy rows ERL_NIF_DIRTY_JOB_CPU_BOUND (:478);
Elixir stubs REQUIRED per registered NIF (cgap_ex.ex:73-89 — the
:74-76 stub-drift note is the standing falsifier class).

New export:
  name: cgap_ingress_digest, arity 1, dirty CPU-bound
  in:   iolist/binary (work-order canonical bytes;
        enif_inspect_iolist_as_binary so iovecs pass free)
  out:  {:ok, hash_bin :: binary-size(32)}  % raw sha256
        | :error

Hashes the ingress bytes, returns the raw digest; the
"sha256:" <> lowercase-hex decoration stays BEAM-side where it
lives (actuation.ex:113-120). Admission-shaped: untrusted bytes
in, typed verdict out, nothing held past the call — the
cgap_ingress_bytes symbol 08b W3 declared MISSING at the
smallest defensible scope. Stub line + one ErlNifFunc row; bump
len 21 -> 22 in the same commit. Size: S.

## W2 Component B — the digest-grammar decision

Grammars: ash_a2a (actuation.ex:113-120) "sha256:" <> hex256_lower
over term_to_binary(Term, [:deterministic]); cgap (object_store.rs
:5-10, dag.rs:839 blake3) raw 32-byte BLAKE3 with KAT courts
(dag.rs:1463).

RECOMMENDATION: Option 1 — canon at the membrane. ash_a2a's
grammar is the LIVE one (xaas + beam4pm receive and verify it);
cgap's BLAKE3 is real but unconsumed (08b W5: no producer). The
membrane bends, not the referent. cgap-beam is zero-external-crate
(lib.rs:3) — sha256 means a hand-rolled impl (~150 lines FIPS
180-4 + KAT court), the exact precedent shape of cgap-blake3
itself. Option 2 (translation NIF) costs more and smuggles a
canonicalization decision inside a "pure mapping" — the W4-3 trap.

Falsifier: AshA2A.Actuation.digest(term) on a fixed corpus must
equal "sha256:" <> hex(cgap_ingress_digest(bytes)) where bytes =
term_to_binary(term, [:deterministic]) — bit-exact for every
element. Secondary: hand-rolled sha256 vs :crypto.hash(:sha256,_)
disagreement on any element blocks the whole edge. Size: M.

## W3 Component C — rf5-ingress-digest-oracle

Template: rf1-dfg-oracle — standalone [[bin]] (Cargo.toml:11-13)
pinning the canonical impl exactly; oracle-io stdin contract
(UTF-8 stdin, serde_json {op,..}, sorted deterministic stdout,
nothing else — rf1 main.rs:11); BEAM twin drives via port, exact
equality.

Court: place in cgap at crates/cgap-beam/oracle/ as standalone
[[bin]] rf5-ingress-digest-oracle (pure hashing, no cgap-beam lib
dep — preserves rf1's separate-pinned-binary property).
Wire: stdin {"op":"ingress_digest","bytes_hex":"..."} -> stdout
{"digest_hex":"<64 lower hex>"}.
Corpus: 64 work-order terms. CORPUS LAW CAUGHT: 0xA20C_0FFE_E5A7_
E000 is a spread-seasoning XOR, NEVER a generator seed
(06-bench-protocol.md:140, levers-L1-L5.md:83; bitslice.rs:687-699
keeps the roles distinct) — generator is SplitMix64 with a
dedicated seed; the constant XORs onto each element. NOTE FOR THE
LEDGER: the CSCB corpus generator (generate_cscb_corpus.py) uses
the constant AS a seed per the RFC charter — a role divergence
vs 06-bench-protocol's law that the next corpus lane must
reconcile.
Procedure: (1) Elixir corpus -> term_to_binary deterministic;
(2) Actuation.digest/1 each -> expected sha256:<64hex> set;
(3) drive rf5 per element, compare hex-exact; (4) second leg: the
same bytes through cgap-blake3 hash/1 stable across runs + KAT
green (legs recorded, never equated); (5) 64/64 or the W2
falsifier fires. Honest restatement: the court proves AGREEMENT
OF GRAMMAR, not equality of hash functions — the W4-5 sentence is
true only under the W2 canon-at-membrane decision. Size: L.

## W4 Loudly out of scope

Authority union type; full CommandBus bridge; BLAKE3 <-> sha256
store binding; any cgap -> ash_a2a dependency direction
(direction is ash_a2a -> cgap-beam only, xaas pattern);
semantic-jira pack; socket trap.

## W5 Sizes

| component | size |
|---|---|
| A: cgap_ingress_digest NIF | S |
| B: sha256-hex canon decision | M |
| C: rf5 differential court | L |

## See Also

audit/cgapprd-08b-crossrepo-referents.md ·
crates/cgap-beam/src/lib.rs:480 · ex/lib/cgap_ex.ex:73-89 ·
ash_a2a/lib/ash_a2a/actuation.ex:113-120 ·
docs/jira/v26.11.2/06-bench-protocol.md:34,140
