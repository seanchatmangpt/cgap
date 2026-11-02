# DXD-04 — FFI drift validator design (Rust / C header / Go cgo)

Standing: PLANNED. Three surfaces must agree on the exported C-ABI:
`crates/cgap-ffi/src/sovereign.rs` + `lib.rs` (#[no_mangle] fns),
`include/cgap.h` (declarations), and `courts/interop-chicago/ffi/`
(cgo wrappers). Goal: a mechanical gate that fails when any two
surfaces drift, and stays silent for ROADMAP-PENDING symbols.

## 1. Symbol extraction per side

### 1.1 Ground truth: nm -gU on the built dylib

IPC-0 proved source grep lies and the linker table does not. From
`docs/jira/v26.11.2/lanes/lane-ipc-00-ground-truth.md` (W2/W3):

- source grep claims `prefilter8` is on the C-ABI; `nm -gU` shows
  NO `cgap_ingress*` symbol — the membrane simply does not exist.
- the real export table is 11 symbols (W3 table): 4 sovereigns
  (`cgap_admit_frame` sov:413, `cgap_eval_bn3` sov:534,
  `cgap_commit_parquet` sov:698, `cgap_verify_p28` sov:293) plus 7
  slot-surface symbols from lib.rs (cgap_i32_batch, cgap_schema_addr,
  cgap_array_addr, cgap_batch_rows, cgap_version, cgap_digest_of)
  and `cgap_p28_invariant_check` which rides the link from
  cgap-p28 trap.rs:82 with no local #[no_mangle] at all.

Therefore the validator's registry of truth is:

    nm -gU <dylib> | awk '{print $3}' | grep '^cgap_' | sort

(dylib from `cargo build --release -p cgap-ffi`; IPC-0 used
`CARGO_TARGET_DIR=/tmp/ipc0-target`, cdylib 615,552 B). `nm -gU`
is the ONLY authoritative side; the other two sides are compared
against it, never against each other.

Darwin note: `nm -gU` = defined external symbols; on Linux use
`nm -D --defined-only` (validator picks by `uname -s`).

### 1.2 Rust side (extracted, not authoritative)

    grep -n '#\[no_mangle\]' -A3 crates/cgap-ffi/src/*.rs
      # then filter lines: pub (unsafe )?extern "C" fn <name>

Plus `#[export_name = "..."]` in xip_tables.rs:143/149
(`cgap_xip_oabi_table`, `cgap_xip_bn3_network`) — cfg-gated, so
they are extracted with the same pattern but matched against the
registry only when the dylib was built with those cfgs on.

Extraction is over the whole crate src tree, not just sovereign.rs:
`cgap_p28_invariant_check` lives in cgap-p28 and rides the link
(IPC-0 W3), so a src-only grep under cgap-ffi would under-report.

### 1.3 Header side

Reuse the wave3_courts parsing approach already landed in
`crates/cgap-ffi/src/sovereign.rs::wave3_courts` (line 1022):
strip block comments (flat scanner, header has no nested
comments), collapse whitespace, then regex

    ^CgapStatus cgap_[a-z_0-9]+ \( [^;]* \) ;

Return-type is currently uniform (`CgapStatus`); pin the return
type in the parse so a `void` drift cannot masquerade as OK.
The court proves the parser is exact for this header via its
M1-M12 mutation corpus; DXD-04 reuses the same parsers verbatim
(plain grep -E on the raw header would false-positive inside the
comment block that mentions symbol names in prose).

### 1.4 Go side

`courts/interop-chicago/ffi/` currently has blake3.go (pure Go,
no cgo). The real cgo preamble lives in
`ingress_court_binding.go` (disclosed ownership divergence in its
header comment). Extract from the cgo preamble `extern`
declarations:

    awk '/import "C"/{exit} /extern/{print}' *.go   # within preamble

plus any `C.cgap_*` call sites. Binding wrappers per symbol:

    grep -n 'C\.cgap_[a-z_0-9]*' courts/interop-chicago/**/*.go

Go extraction spans ALL .go files in interop-chicago (preamble +
bindings + tests), since cgo cannot live in _test.go (golang/go#8601).

## 2. Registry, diff, and the ROADMAP carve-out

### 2.1 Registry construction

Expected set R = union of:
- the 11 nm-verified symbols (IPC-0 W3, hard-coded as the baseline
  with a "why" comment per symbol),
- every symbol extracted from header and Rust and Go sources.

A symbol is REQUIRED when it appears in nm output; it is
DECLARED when it appears in header/Rust/Go. Drift cases:

| case | meaning | verdict |
|---|---|---|
| nm has, header lacks | header behind impl | FAIL |
| header has, nm lacks | declared but unexported | FAIL |
| nm has, Rust grep lacks | export without src owner | FAIL |
| nm has, Go lacks | no cgo binding | FAIL |
| in none of the three, in registry | registry stale | FAIL |

### 2.2 ROADMAP-PENDING symbols (the NO-GO table)

IPC-0 W5 marks CTQ-02 (`cgap_ingress_bytes`), CTQ-04
(`cgap_radix_join`), and shm (`cgap_shm_*`) NO-GO: these are
expected absences, not drift. Design: a versioned registry file

    courts/interop-chicago/ffi/ffi_symbol_registry.toml

with three sections:

    [symbols]        # name -> {rust, header, go} expectations
    [roadmap]        # pending symbols, expected-missing
    [structures]     # struct pins (sec 3)

Roadmap section entries carry the NO-GO citation
(lane-ipc-00-ground-truth.md W5 suite + CTQ). The gate rule:
a symbol in `[roadmap]` that is missing from nm/header/Go is
reported as `PENDING` in the output and does NOT fail the gate;
if it APPEARS (someone lands `cgap_ingress_bytes` without
removing its roadmap row), that IS a gate failure —
stale-registry drift, mirrored to the W4 finding that grep
claimed an export that did not exist. This inverts cleanly:
absence is allowed, unregistered presence is not.

Chosen over a header `/* allowlist */` comment: comments in
cgap.h are normative prose (ABI-freeze contract text); burying
gate bookkeeping in them invites the exact drift this gate
exists to catch. The TOML file is machine-read, diff-able, and
lives beside the Go bindings it serves.

## 3. Struct-layout verification

Reuse the landed FIX-03 static-assert pattern, both legs:

### 3.1 C leg

`crates/cgap-ffi/tests/arrow_offset_pins.rs` embeds a versioned
C pin block (static asserts on offsetof/sizeof/_Alignof) and
cc-compiles it against `include/` — that court is the model.
For DXD-04 the pin block targets `cgap_frame_view` from
include/cgap.h (the header already carries its own C11 pins,
LP64 24 B / ILP32 16 B, offsetof per field — lines 90-128 of
cgap.h). The validator:

1. stages the header's own pin block (extracted from the
   `#if C11` region) into /tmp,
2. cc -std=c11 -I <mutated-include-dir> compiles it,
3. expects exit 0 on the in-tree header.

This proves the C declarations still lay out identically to the
pins; a retype or reorder fails the compile. When cgap.h gains a
struct, the pin block in the TOML `[structures]` section gains a
row; rows carry LP64 + ILP32 offsets.

### 3.2 Rust leg

Same file (`arrow_offset_pins.rs`) shows the Rust twin: const
asserts over the real #[repr(C)] struct plus a runtime mirror,
and the wrong-pin rustc twin proving the pins carry bits. DXD-04
adds a twin pinning `CgapFrameView` (size/align/offsets must
equal the header's numbers). If sovereign.rs's CgapFrameView and
the header disagree, the twin fails to compile.

### 3.3 Cross-leg equality

The validator asserts header-declared offsets == Rust offsets ==
TOML rows, one struct at a time. Go is excluded here: cgo sees
the layout through the C preamble typedef (it compiles the
preamble with cc), so the C leg covers it transitively; the Go
side needs only the preamble typedef to match cgap.h, checked
textually (field list + types) since cgo has no offsetof.

Go preamble check: normalize both sides (strip comments, collapse
ws — same helpers as wave3_courts) and require the typedef field
sequence to be a prefix-preserving match of the header's
cgap_frame_view. Note the disclosed divergence: today the Go
preamble REDEFINES cgap_frame_view inline instead of #include-ing
the header; the validator flags this as WARN (works, but a third
copy of the layout exists) and the TOML can pin
`go_preamble_includes_header = true` later.

## 4. Failure output format

One line per symbol, aligned table, exit 1 on any FAIL row:

    DXD-04 FFI drift: 12 symbols checked, 1 FAIL, 1 PENDING
    symbol                   nm(dylib) header rust  go    verdict
    cgap_admit_frame         present   present present present OK
    cgap_ingress_bytes       absent    absent  absent  absent PENDING (W5 CTQ-02)
    cgap_radix_join          absent    absent  absent  absent PENDING (W5 CTQ-04)
    cgap_shm_ring_attach     absent    absent  present absent  FAIL header-only
    cgap_eval_bn3            present   absent  present present  FAIL header-missing

Exit codes: 0 = all OK/PENDING; 1 = any FAIL; 2 = harness error
(dylib not built, cc/rustc/go missing) — distinct so CI can
distinguish "gate tripped" from "gate could not run".

## 5. Anti-vacuity: the /tmp header-copy mutation

Spec (concrete, mirroring arrow_offset_pins.rs
c_pin_flip_on_header_mutation):

Fixture:
1. cp include/cgap.h /tmp/dxd04-mut/include/cgap.h
2. Apply ONE drift: delete the cgap_eval_bn3 declaration block
   (from `CgapStatus cgap_eval_bn3` through `;`).
3. Assert the mutation was a no-op guard:
   `assert mutated != original` (the court prints VACUOUS and
   fails if replacen found no match — same pattern as the
   FIX-03 court's assert_ne! on header vs mutated).
4. Build the dylib normally (unmutated); run the validator with
   CGAP_HEADER=/tmp/dxd04-mut/include/cgap.h override.
5. EXPECT: exit 1, and the per-symbol table shows
   `cgap_eval_bn3  present  absent  present  present  FAIL header-missing`.
6. Assert the exact row appears in stdout (grep), not just a
   nonzero exit — a validator that fails loudly but generically
   is half-vacuous.

Negative control: run the validator with the in-tree header;
must exit 0. Positive+negative together prove the gate carries
bits in both directions.

Second mutation (struct leg): stage a /tmp copy of cgap.h with
`cgap_frame_view.payload_len` retyped `size_t -> uint32_t`, run
the cc pin block against only the mutated include dir (the
quoted-form #include resolution trick from arrow_offset_pins.rs,
c_pin_flip_on_header_mutation), assert compile FAILS with an
offsetof/static-assert diagnostic mentioning cgap_frame_view.

Third mutation (Go leg): delete the `extern ... cgap_admit_frame`
line from a /tmp copy of the preamble, run the Go extractor,
assert the Go column flips to absent and the verdict becomes
FAIL go-missing.

All mutations touch /tmp copies only; the shared tree is never
modified (same fence as FIX-03: "Courts mutate /tmp copies only").

## 6. Fences

- Read-only wrt crates/ and include/; the validator writes only
  /tmp staging + its own registry TOML (versioned, in-repo).
- Does not touch zero_string_gate.py or boundary_gate.py.
- The gate is additive to the landed wave3_courts header court:
  wave3 verifies header-internal consistency + ABI version; DXD-04
  adds the cross-surface symbol matrix, keyed on nm as ground
  truth per IPC-0.
