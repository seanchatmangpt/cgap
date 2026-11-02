# cgapprd-08b-crossrepo-referents

CGAP-PRD-8 companion: the four out-of-tree referent repos
(~/ash_a2a, ~/xaas, ~/beam4pm, ~/smalt), read-only, 2026-11-02.

## W0 Headline

The PRD's "AshA2A" clause has REAL referents — entirely outside cgap,
with the referent chain running opposite the PRD's grammar:

- ash_a2a really does project work orders into content-addressed
  identities: AshA2A.Hilt.WorkOrder.identity_digest/1
  (lib/ash_a2a/hilt/work_order.ex:198-204) via AshA2A.Actuation.digest
  (SHA-256 over :erlang.term_to_binary, actuation.ex:113-120).
- xaas and beam4pm are LIVE downstream consumers (pinned git refs in
  both mix.exs).
- cgap HAS a real BEAM membrane (crates/cgap-beam NIF + ex/lib/
  cgap_ex.ex) that none of the four consumes.
- Zero hits for "cgap" in any of the four repos' source/deps/locks
  (grep filtered _build/deps/tmp).

## W1 Repo identities

### ash_a2a
Elixir Spark.Dsl.Extension exposing Ash actions as A2A-protocol agent
skills (JSON-RPC/HTTP+SSE, gRPC, A2A v1.0 REST; README:5-16; SA2A
codename :18-21). PRD-relevant surfaces, all real:
1. Work-order projection: Hilt.WorkOrder — "not authority and never
   performs DO" (:7-9); bind_command places the content-addressed
   identity into command metadata (:14-16, impl :210-230);
   @enforce_keys digests (subject/candidate/bounds/evidence/
   falsifier :44-56); stale_graph_identity refusal (:31).
2. Authority: AshA2A.Authority (authority.ex:5) with broker/, lease,
   grant, two_port_gate, security_preflight; refusal taxonomy
   -> :refused_authority (:172-176); receipted CommandBus with
   replay-safe receipts (README:44-50). Real admission pipeline:
   semantic/{admission,admission_pipeline,admission_hash,
   admission_refusal,admission_standing}.ex.
3. Semantic-JIRA pack: NOT FOUND — no jira-named module; RDF shapes
   are SA2A-doctrine (semantic/*.ttl).
4. No FFI/shm of its own: native/ = graphlaw_host (wasmtime) +
   hddl_cli — hosts, not membranes.

### xaas
Elixir/Phoenix Ash platform: 19 Ash domains, Ash.Reactor actuation
(admit -> do -> receipt, README:5-12,35-40). Consumer precedent:
mix.exs:116-120 pins {:ash_a2a, git: ref 86214551..., override: true};
consumes via the A2a domain (lib/xaas/a2a/). Pins sibling path deps
../ash_graphlaw, ../ash_surface, ../ash_affidavit (mix.exs:143-145) —
the established cross-repo Elixir wiring pattern. No cgap dep.

### beam4pm
BEAM-first process-mining substrate, ggen-manufactured from RDF
(README:3-12). Consumer precedent same shape: mix.exs:133-148
ash_a2a_dep — Hex cap ~> 26.9.30 under BEAM4PM_HEX_PUBLISH=1, else
git ref 99280b85...; lockstep rule :120-124. Live use:
beam4pm_a2a_agent.ex, beam_pm/sa2a/replan_consumer.ex,
beam4pm_research_erc.ex. FFI court is large and real: native/ =
ferroplan, graphlaw, oracle-io, rf1..rf4 conformance/OCel oracles,
rust4pm-wasm, tract-wasm, petgraph-wasm — differential Rust oracles
behind BEAM, the closest architectural precedent for "cgap as
oracle/membrane". No cgap dep.

### smalt
Pure-Elixir BEAM-native Azure emulator (README:3-10); honest standing
block (:12-23, e.g. PQC GAP_TYPED). No ash_a2a dep (pin-mirror
comment mix.exs:176). shm/mmap hits are AMQP/storage internals only —
least cgap-relevant of the four.

## W2 The missing link both audits half-saw

cgap already ships a BEAM NIF: crates/cgap-beam/src/{lib.rs,shm.rs,
spsc_ring.rs,resbin.rs,ops_scan.rs,ops_persist.rs,...} with
ex/lib/cgap_ex.ex loading libcgap_beam.dylib via @on_load
(cgap_ex.ex:14-63) and stubs cgap_new/len/get/free/version,
cgap_new_i64, cgap_get_i64, cgap_stats, cgap_new_fast,
cgap_envelope_new/len, cgap_batch_binary (cgap_ex.ex:75-89). The
protocol surface EXISTS in cgap — unconsumed by every repo the
operator named. The 08 audit's "zero in-tree wiring" was correct for
cgap's tree AND is now provably correct for all four referent trees:
the wiring is bidirectionally absent; only the ingredients exist on
both sides.

## W3 Clause reconciliation (supersedes 08 W1 where marked)

| # | clause | verdict after cross-repo read |
|---|---|---|
| 1 | work orders -> coordinate identities | REAL REFERENT, out-of-tree: Hilt.WorkOrder identity_digest + bind_command. CGAP SIDE ABSENT: no work-order ingest; clause-1 substrate (dag.rs, object_store BLAKE3) has no producer speaking this shape. ASPIRATIONAL with named referent |
| 2 | authority at the ingress ring | DUAL REFERENT, two unmerged rings: ash_a2a Authority/CommandBus + semantic admission (BEAM-side authority) vs cgap_admit_frame (sovereign.rs:413, Rust ring). No protocol joins them. ASPIRATIONAL with named referents |
| 3 | AshA2A pack consumption | UPGRADED vendor-claim to REAL-BUT-INVERTED: ash_a2a real, consumed by xaas + beam4pm; what does not exist is cgap consuming ash_a2a or vice versa |
| 4 | zero-copy vector I/O | UNCHANGED VERIFIED; note cgap-beam NIF + shm.rs/spsc_ring.rs give a real BEAM membrane the 08 audit did not cite |
| 5 | socket trap | UNCHANGED ASPIRATIONAL (smalt closest — real TCP listeners — but an Azure emulator, no dialer-Control analogue) |

## W4 What wiring would require (design-level)

1. Dependency direction: cgap Rust, ash_a2a Elixir — the honest
   architecture is cgap-beam NIF becomes a package; ash_a2a (or a
   thin cgap_ingress adapter) depends on it. Direction:
   ash_a2a -> cgap-beam, never cgap -> ash_a2a (xaas pattern).
2. Protocol surface: cgap-beam must export an ingress membrane at
   the NIF boundary — the cgap_ingress_bytes symbol (08 W3 MISSING)
   exposed as a NIF taking an iovec, returning the typed verdict.
   Today's NIF is alloc/scan/envelope-shaped, not admission-shaped.
3. Projection contract: ash_a2a computes "sha256:" <> hex over
   deterministic term_to_binary; cgap computes 32-byte BLAKE3.
   These do not interoperate by accident: cgap adopts a sha256-hex
   canonicalization at the membrane, or a translation NIF is
   specified. A real design decision the PRD never states.
4. Authority mapping: a Refusal union spanning both
   (:refused_authority <-> cgap typed refusals) is the minimum.
5. Test referent: beam4pm's rf1-rf4 oracle pattern (BEAM impl vs
   canonical Rust process_mining 0.6.2, Cargo.toml:5,17) is the
   template: a differential oracle court proving the BEAM work-order
   digest path and the cgap BLAKE3 ring agree on a fixed corpus.

## W5 Honest vocabulary

- Real + landed: ash_a2a work-order projection, authority ring,
  command bus; xaas + beam4pm consumption; cgap-beam NIF; beam4pm
  oracle-FFI court.
- Real but unconsumed: the cgap-beam membrane; cgap BLAKE3 substrate
  (no producer).
- Absent everywhere: the cgap<->ash_a2a edge in either direction;
  shared digest grammar; NIF-level ingress membrane; semantic-jira
  pack; socket-trap referent.
- The PRD §4 sentence names an ECOSYSTEM (ash_a2a + two consumers +
  cgap's substrate), not a wired system. Every noun now has a
  file:line referent; the verb ("projects ... into") has no
  implementation spanning the two trees.

## See Also

audit/cgapprd-08-boundaries.md (companion) ·
ash_a2a/lib/ash_a2a/hilt/work_order.ex · xaas/mix.exs ·
beam4pm/mix.exs · crates/cgap-beam/src/lib.rs
