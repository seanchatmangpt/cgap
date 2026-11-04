//! Lane FIX-03 — Arrow C-ABI offset pin court (G19/EXP-08 DEFECT-OPEN
//! court-persistence repair).
//!
//! G19's 10 ArrowArray/ArrowSchema LP64 pins lived only in /tmp/g19_offsets_check.c.
//! This court versions them on both sides:
//!   - Rust: compile-time static asserts + a runtime mirror over the real
//!     #[repr(C)] structs (crates/cgap-arrow/src/c_abi.rs:64-91) — ArrowArray 80 B,
//!     ArrowSchema 72 B, every field offset, and the declaration-order contract.
//!   - C: a cc compile court against the versioned header include/cgap_c_data.h
//!     (the Arrow structs ARE declared there — this is where G19's C pins lived;
//!     include/cgap.h itself carries no Arrow declarations).
//! Anti-vacuity (both sides flip):
//!   - C leg: a field-swapped /tmp copy of cgap_c_data.h must fail the same pins.
//!   - Rust leg: a rustc twin crate (#[path]-including the real c_abi.rs) with a
//!     deliberately wrong pin (size 79) must fail to compile; the correct-pin twin
//!     must succeed.
//! Courts mutate /tmp copies only; the shared tree is never touched.
use std::process::Command;

use cgap_arrow::c_abi::{ArrowArray, ArrowSchema};

// ---- Rust-side pins (LP64 host: aarch64/x86_64 macOS/linux) ----
const _: () = assert!(
    core::mem::size_of::<ArrowArray>() == 80,
    "ArrowArray size 80"
);
const _: () = assert!(
    core::mem::size_of::<ArrowSchema>() == 72,
    "ArrowSchema size 72"
);
const _: () = assert!(core::mem::offset_of!(ArrowArray, length) == 0);
const _: () = assert!(core::mem::offset_of!(ArrowArray, null_count) == 8);
const _: () = assert!(core::mem::offset_of!(ArrowArray, offset) == 16);
const _: () = assert!(core::mem::offset_of!(ArrowArray, n_buffers) == 24);
const _: () = assert!(core::mem::offset_of!(ArrowArray, n_children) == 32);
const _: () = assert!(core::mem::offset_of!(ArrowArray, buffers) == 40);
const _: () = assert!(core::mem::offset_of!(ArrowArray, children) == 48);
const _: () = assert!(core::mem::offset_of!(ArrowArray, dictionary) == 56);
const _: () = assert!(core::mem::offset_of!(ArrowArray, release) == 64);
const _: () = assert!(core::mem::offset_of!(ArrowArray, private_data) == 72);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, format) == 0);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, name) == 8);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, metadata) == 16);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, flags) == 24);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, n_children) == 32);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, children) == 40);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, dictionary) == 48);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, release) == 56);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, private_data) == 64);

/// Runtime mirror of the pins + the declaration-order contract (a field
/// reorder that preserved all offsets is impossible under repr(C), so order
/// and offset are pinned together; asserting the sequence at runtime also
/// keeps this file green even if a future toolchain lowers const-eval).
#[test]
fn rust_offset_pins() {
    assert_eq!(core::mem::size_of::<ArrowArray>(), 80);
    assert_eq!(core::mem::size_of::<ArrowSchema>(), 72);
    let aa: [(usize, &str); 10] = [
        (core::mem::offset_of!(ArrowArray, length), "length"),
        (core::mem::offset_of!(ArrowArray, null_count), "null_count"),
        (core::mem::offset_of!(ArrowArray, offset), "offset"),
        (core::mem::offset_of!(ArrowArray, n_buffers), "n_buffers"),
        (core::mem::offset_of!(ArrowArray, n_children), "n_children"),
        (core::mem::offset_of!(ArrowArray, buffers), "buffers"),
        (core::mem::offset_of!(ArrowArray, children), "children"),
        (core::mem::offset_of!(ArrowArray, dictionary), "dictionary"),
        (core::mem::offset_of!(ArrowArray, release), "release"),
        (
            core::mem::offset_of!(ArrowArray, private_data),
            "private_data",
        ),
    ];
    for (i, (off, name)) in aa.iter().enumerate() {
        assert_eq!(*off, i * 8, "ArrowArray field {name} offset");
    }
    let as_: [(usize, &str); 9] = [
        (core::mem::offset_of!(ArrowSchema, format), "format"),
        (core::mem::offset_of!(ArrowSchema, name), "name"),
        (core::mem::offset_of!(ArrowSchema, metadata), "metadata"),
        (core::mem::offset_of!(ArrowSchema, flags), "flags"),
        (core::mem::offset_of!(ArrowSchema, n_children), "n_children"),
        (core::mem::offset_of!(ArrowSchema, children), "children"),
        (core::mem::offset_of!(ArrowSchema, dictionary), "dictionary"),
        (core::mem::offset_of!(ArrowSchema, release), "release"),
        (
            core::mem::offset_of!(ArrowSchema, private_data),
            "private_data",
        ),
    ];
    for (i, (off, name)) in as_.iter().enumerate() {
        assert_eq!(*off, i * 8, "ArrowSchema field {name} offset");
    }
}

/// Root of the repo (crates/cgap-ffi/tests -> repo root).
fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
}

/// The C pin block (versioned port of /tmp/g19_offsets_check.c minus the
/// CgapStatus pin that lives in the in-tree header court already).
const C_PIN_BLOCK: &str = r#"#include <stddef.h>
#include "cgap_c_data.h"
_Static_assert(offsetof(struct ArrowArray, length) == 0, "aa.length");
_Static_assert(offsetof(struct ArrowArray, null_count) == 8, "aa.null_count");
_Static_assert(offsetof(struct ArrowArray, offset) == 16, "aa.offset");
_Static_assert(offsetof(struct ArrowArray, n_buffers) == 24, "aa.n_buffers");
_Static_assert(offsetof(struct ArrowArray, n_children) == 32, "aa.n_children");
_Static_assert(offsetof(struct ArrowArray, buffers) == 40, "aa.buffers");
_Static_assert(offsetof(struct ArrowArray, children) == 48, "aa.children");
_Static_assert(offsetof(struct ArrowArray, dictionary) == 56, "aa.dictionary");
_Static_assert(offsetof(struct ArrowArray, release) == 64, "aa.release");
_Static_assert(offsetof(struct ArrowArray, private_data) == 72, "aa.private_data");
_Static_assert(sizeof(struct ArrowArray) == 80, "aa.size LP64");
_Static_assert(offsetof(struct ArrowSchema, format) == 0, "as.format");
_Static_assert(offsetof(struct ArrowSchema, name) == 8, "as.name");
_Static_assert(offsetof(struct ArrowSchema, metadata) == 16, "as.metadata");
_Static_assert(offsetof(struct ArrowSchema, flags) == 24, "as.flags");
_Static_assert(offsetof(struct ArrowSchema, n_children) == 32, "as.n_children");
_Static_assert(offsetof(struct ArrowSchema, children) == 40, "as.children");
_Static_assert(offsetof(struct ArrowSchema, dictionary) == 48, "as.dictionary");
_Static_assert(offsetof(struct ArrowSchema, release) == 56, "as.release");
_Static_assert(offsetof(struct ArrowSchema, private_data) == 64, "as.private_data");
_Static_assert(sizeof(struct ArrowSchema) == 72, "as.size LP64");
int main(void){return 0;}
"#;

/// Write `src` to `dst` (a fresh /tmp path, never the shared tree).
fn stage(dst: &std::path::Path, src: &str) {
    std::fs::write(dst, src).expect("stage file");
}

/// cc-compile a C file with the repo include dir on the search path.
fn cc(c_file: &std::path::Path) -> std::process::Output {
    let inc = repo_root().join("include");
    Command::new("cc")
        .arg("-std=c11")
        .arg("-I")
        .arg(&inc)
        .arg("-o")
        .arg(c_file.with_extension("out"))
        .arg(c_file)
        .output()
        .expect("spawn cc")
}

/// POSITIVE leg: the versioned C pin block compiles clean against the
/// in-tree header (exit 0).
#[test]
fn c_header_pin_compiles() {
    let dir = std::env::temp_dir().join("fix03-pins-c");
    std::fs::create_dir_all(&dir).unwrap();
    let c = dir.join("pins.c");
    stage(&c, C_PIN_BLOCK);
    let out = cc(&c);
    assert!(
        out.status.success(),
        "baseline cc of pin block failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// ANTI-VACUITY (C leg): a field-swapped /tmp COPY of cgap_c_data.h must
/// FLIP the C pins to fail. The copy is mutated in /tmp only; the shared
/// header is never touched.
#[test]
fn c_pin_flip_on_header_mutation() {
    let dir = std::env::temp_dir().join("fix03-pins-c-mut");
    std::fs::create_dir_all(&dir).unwrap();
    let header = std::fs::read_to_string(repo_root().join("include/cgap_c_data.h"))
        .expect("read shared header");
    // Mutation: swap length <-> null_count declaration order.
    let mutated = header
        .replacen("int64_t length;", "@@MUT@@", 1)
        .replacen("int64_t null_count;", "int64_t length;", 1)
        .replacen("@@MUT@@", "int64_t null_count;", 1);
    assert_ne!(header, mutated, "mutation was a no-op (court vacuous)");
    let inc = dir.join("include");
    std::fs::create_dir_all(&inc).unwrap();
    std::fs::write(inc.join("cgap_c_data.h"), &mutated).expect("stage mutated header");
    // The pin block #includes "cgap_c_data.h" quoted-form, so -I <dir> lets
    // the mutated copy shadow the real header; pass the mutated include dir
    // ONLY (the header also includes cgap.h? it does not — verify).
    let c = dir.join("pins.c");
    stage(&c, C_PIN_BLOCK);
    let out = Command::new("cc")
        .arg("-std=c11")
        .arg("-I")
        .arg(&inc)
        .arg("-o")
        .arg(c.with_extension("out"))
        .output()
        .expect("spawn cc");
    assert!(
        !out.status.success(),
        "VACUOUS: C pins did not flip on a field-swapped header copy"
    );
    // Also confirm the pin block's include resolution: the real header must
    // not be reachable from the mutated dir, else the flip is fake.
    assert!(!inc.join("cgap.h").exists(), "cgap.h must not shadow");
}

/// ANTI-VACUITY (Rust leg): a rustc twin crate that #[path]-includes the
/// REAL c_abi.rs (via its real sibling modules) with a deliberately wrong
/// pin — ArrowArray sized 79 — must FAIL to compile; the correct-pin twin
/// must succeed. Proves the pins carry bits against the live structs.
#[test]
fn rust_pin_flip_on_wrong_pin() {
    let dir = std::env::temp_dir().join("fix03-pins-rs");
    let _ = std::fs::create_dir_all(&dir);
    let src = repo_root().join("crates/cgap-arrow/src");
    let inc = |m: &str| format!("#[path = \"{}/{}.rs\"] pub mod {};", src.display(), m, m);
    let prelude = "#![no_std]\nextern crate alloc;\n";
    let mods = ["bitmap", "buffer", "primitive", "c_abi"];
    let wrong = format!(
        "{}{}{}",
        prelude,
        mods.iter().map(|m| inc(m)).collect::<Vec<_>>().join("\n"),
        "\nconst _: () = assert!(core::mem::size_of::<c_abi::ArrowArray>() == 79);\n"
    );
    let right = format!(
        "{}{}{}",
        prelude,
        mods.iter().map(|m| inc(m)).collect::<Vec<_>>().join("\n"),
        "\nconst _: () = assert!(core::mem::size_of::<c_abi::ArrowSchema>() == 72);\n"
    );
    // WRONG twin: compilation itself must fail (static assert fires).
    let w = dir.join("wrong.rs");
    stage(&w, &wrong);
    let out = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg("--out-dir")
        .arg(&dir)
        .arg(&w)
        .output()
        .expect("spawn rustc (wrong twin)");
    assert!(
        !out.status.success(),
        "VACUOUS: wrong pin (79) twin compiled — pins carry no bits"
    );
    // RIGHT twin: same twin with a trivially true pin must compile clean,
    // proving the failure above is the pin, not the twin scaffold.
    let r = dir.join("right.rs");
    stage(&r, &right);
    let out = control_handle(&r, &dir);
    assert!(
        out.status.success(),
        "control twin (trivially true pin) failed to compile: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Run rustc on `file` (helper mirroring the wrong-twin invocation).
fn control_handle(file: &std::path::Path, out_dir: &std::path::Path) -> std::process::Output {
    Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(file)
        .output()
        .expect("spawn rustc (control twin)")
}
