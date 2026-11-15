# Lane Ra3 (Raise) - KD-4 oabi-lookup / A19 reachability

Standing: BLOCKED (verdict only; KD-4 stays OPEN). No source edited, no git writes.

## Question
Has A19 slot resolution landed so `crates/cgap-ffi/src/oabi_lookup.rs` can gain a real implementation (KD-4 close condition)?

## Evidence (read this session)
- A19 spec `docs/jira/v26.11.1/lanes/A19-libcgap-ffi-xip.md:47,57-60` still fixes the boundary at exactly four verbs; a fifth verb "belongs in a lane, not the ABI". No A19 revision defines slot resolution. Only one A19 lane file exists (`ls docs/jira/v26.11.1/lanes | grep -i a19`).
- The four sovereigns HAVE landed since the a19-finish-receipt (which recorded DoD 5 BLOCKED): `crates/cgap-ffi/src/sovereign.rs:298 cgap_verify_p28`, `:424 cgap_admit_frame`, `:552 cgap_eval_bn3`, `:722 cgap_commit_parquet`. cgap-ffi/Cargo.toml now has dep edges to cgap-core/gate/bn3/blake3/parquet/p28/crypto/aoq. This retires only ground 4 (structural dep edge) of the in-file decision record, and only partly: there is still NO cgap-bont dep edge (grep "bont" in crates/cgap-ffi/Cargo.toml: 0 hits).
- Grounds 1-3 of the record (four-verb law, fifth-verb rule, no owner lane for a slot-lookup body) are unchanged. `sovereign.rs:365` still points at `oabi_lookup.rs::OABI_LOOKUP_UNSUPPORTED`.
- `include/cgap.h` has no oabi lookup prototype (only CGAP_ERR_OABI_MISMATCH, frame-rule-count commentary).
- ontology-packs/{fibo,cloud}: `.oabi` files + `ontology-packs/cloud/src/lib.rs:324 resolve_equivalence(alias_slot)` exist, but that is pack-local Rust API, not an A19 boundary surface; it does not satisfy "A19 lands slot resolution".
- Surface-freeze court in oabi_lookup.rs still enforces no `oabi` export; not run this lane (no source change, verdict-only).

## Verdict
BLOCKED-on-A19 stands. Landing of the four sovereigns is NOT landing of slot resolution. A19 did not clearly land; per dispatch, no diff proposed. Not run: any build/test (UNVERIFIED that court is green today).

## Unblock path (for the coordinator)
1. Amend A19 (spec revision) to either add a verb (breaking the "exactly four" ADR) or define slot resolution inside `cgap_eval_bn3` semantics, with an owner lane.
2. Only then: add cgap-bont dep edge, replace the record, and update the surface-freeze court (its detector currently refuses any `oabi` export; it would need an intentional flip plus anti-vacuity mutation).
3. Update KD-4 row and A10 T5 per A10 section 10.

## Fences
Diff touches only this receipt; no existing gate/court can regress.

## Anti-vacuity
Verdict claims are falsifiable by: grep of A19 spec for a fifth verb / slot-resolution section (none found); a `#[no_mangle]` oabi symbol in cgap-ffi (none; the in-file court flips on one).

## Commands
Read/grep/ls only, all exit 0. Divergences: KD-4 recount note says oabi_lookup.rs:1-31 is the record; the record now spans lines 1-50 (cosmetic).
