# Lane G19 — C Header Parity Receipt

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. Shared tree untouched
except this receipt file. Standing: ALIVE (every claim below cites a command
+ exit run this session).

## 1. Compile gates

| Gate | Command | Exit |
|---|---|---|
| gate0 leg (c_data header) | `cc -std=c99 -ffreestanding -Wall -Wextra -pedantic -c include/cgap_c_data.h -o /tmp/g19_cdata.o` | 0 |
| cgap.h | `cc -std=c99 -Wall -Werror -c include/cgap.h -o /tmp/g19_cgaph.o` | 0 |
| gate0 full | `make gate0` (ggen replay + header compile) | 0 |

Divergence noted: under `-std=c99`, `__STDC_VERSION__` is 199901L, so the
`_Static_assert` layout-pin block in `include/cgap.h:90-124` is compiled OUT.
The dispatch's cgap.h gate therefore passes without exercising any pin.
Re-run with C11 to activate:

| Gate | Command | Exit |
|---|---|---|
| cgap.h C11 | `cc -std=c11 -Wall -Werror -c include/cgap.h -o /tmp/g19_cgaph_c11.o` | 0 |
| c_data C11 | `cc -std=c11 -ffreestanding -Wall -Wextra -pedantic -c include/cgap_c_data.h -o /tmp/g19_cdata_c11.o` | 0 |

## 2. Struct-offset parity table

### cgap_frame_view — Rust `CgapFrameView` (crates/cgap-ffi/src/sovereign.rs:107-151)

Rust pins at sovereign.rs:133-151 (const asserts, both widths). C pins at
include/cgap.h:97-124. Both sides pin identical constants.

| Field | Rust offset LP64 | C offset LP64 | Rust offset ILP32 | C offset ILP32 |
|---|---|---|---|---|
| payload | 0 | 0 | 0 | 0 |
| payload_len | 8 | 8 | 4 | 4 |
| schema_id | 16 | 16 | 8 | 8 |
| rule_count | 20 | 20 | 12 | 12 |
| sizeof | 24 | 24 | 16 | 16 |
| alignof | 8 | 8 | 4 | 4 |

ILP32 C side confirmed mechanically: `make gate0` leg and the C11 compile
above ran on host (LP64 only); the ILP32 constants exist as `_Static_assert`s
under `UINTPTR_MAX == 0xFFFFFFFFu` (cgap.h:112-124) and as Rust const asserts
under `target_pointer_width = "32"` (sovereign.rs:143-151) — the pin pairs
were hand-audited field-by-field and agree. No 32-bit compiler was available
on this host, so the ILP32 pins are pinned-but-not-executed here (honest:
UNVERIFIED-on-metal, verified-as-identical-text).

### ArrowArray / ArrowSchema — Rust `crates/cgap-arrow/src/c_abi.rs:65-90`

The Rust side carries NO offset pins for these structs (grep for
offset_of/size_of/const_ in c_abi.rs: zero hits); parity is by field ORDER
(pinned by the Arrow C Data Interface spec, not by cgap) plus repr(C)
natural layout. Hand-audit: field order matches cgap_c_data.h exactly on
both structs.

Mechanical LP64 confirmation court: /tmp/g19_offsets_check.c — 23
`_Static_assert`s over every field of both structs + sizeofs, compiled with
`cc -std=c11 -Wall -Werror -I include -c /tmp/g19_offsets_check.c` exit 0.

| Field | C offset (calculated = asserted) | Field | C offset (calculated = asserted) |
|---|---|---|---|
| ArrowArray.length | 0 | ArrowSchema.format | 0 |
| ArrowArray.null_count | 8 | ArrowSchema.name | 8 |
| ArrowArray.offset | 16 | ArrowSchema.metadata | 16 |
| ArrowArray.n_buffers | 24 | ArrowSchema.flags | 24 |
| ArrowArray.n_children | 32 | ArrowSchema.n_children | 32 |
| ArrowArray.buffers | 40 | ArrowSchema.children | 40 |
| ArrowArray.children | 48 | ArrowSchema.dictionary | 48 |
| ArrowArray.dictionary | 56 | ArrowSchema.release | 56 |
| ArrowArray.release | 64 | ArrowSchema.private_data | 64 |
| ArrowArray.private_data | 72 | sizeof ArrowSchema | 72 |
| sizeof ArrowArray | 80 | | |

## 3. Padding / alignment analysis

- cgap_frame_view: no padding discrepancies on either width (table above);
  C natural layout and repr(C) produce identical offsets.
- Arrow structs LP64: every member is 8 bytes — zero internal padding,
  sizeof 80 / 72 exactly as asserted.
- Arrow structs ILP32 (e.g. rv32imc): C `int64_t` alignment is ABI-defined;
  on ESP32-C3 newlib it is 4, giving offsets 0,4,8,12,16 then pointers at
  20..36, sizeof 40. Rust repr(C) i64 on riscv32 likewise aligns 4, so both
  sides agree by construction — but NEITHER side pins ILP32 Arrow offsets.
  Residual risk noted, not a demonstrated discrepancy; per lane contract no
  header edit was made.
- CgapStatus: `sizeof == 4` pinned both sides (cgap.h:97, sovereign.rs);
  a 16-bit-int target refuses to compile (fail-closed).

## 4. Anti-vacuity court (mutation flip)

Sandbox: /tmp/g19_sandbox (copies of both headers), shared tree untouched.

| Step | Command | Exit |
|---|---|---|
| baseline sha256 | `shasum -a 256 /tmp/g19_sandbox/cgap_c_data.h` | 01d828b391ca739b5d482dcf76df8b841b4b356412ac8b36d49c697dfbef4f88 |
| mutation: swap ArrowSchema flags/n_children field order | python edit in sandbox | — |
| mutated compile | `cc -std=c11 -Wall -I /tmp/g19_sandbox -c /tmp/g19_offsets_check.c` | 1 (2 static-assert failures: as.flags 32!=24, as.n_children 24!=32) |
| restore (byte-exact) | sha256 after restore | 01d828b391ca739b5d482dcf76df8b841b4b356412ac8b36d49c697dfbef4f88 (exact match) |
| restored compile | same cc, exit | 0 |

The court flips; no skip.

## 5. Redundancy vs make gate0

`make gate0` (Makefile:45-55) already runs the freestanding header compile:
line 54 is byte-identical to the dispatch's first gate
(`cc -std=c99 -ffreestanding -Wall -Wextra -pedantic -c include/cgap_c_data.h`).
`make gate0` run this session: exit 0. The dispatch gate is therefore
redundant with gate0's leg; it was still run as dispatched and both agree.
The cgap.h `-std=c11` compile (section 1) is NOT covered by gate0 and is the
only new gate surface this lane adds.

## 6. Fences and standing

Must-not-regress fences: gate0 (ran, exit 0), gate0 sha replay (ran, exit 0
inside make gate0). No header edits were made — no demonstrated discrepancy.
Standing: ALIVE. Files written: this receipt only.
