//! Lane E13 — C header struct-alignment court (F8 additive remainder).
//!
//! Asserts the Rust #[repr(C)] twins (crates/cgap-arrow/src/c_abi.rs and
//! crates/cgap-ffi/src/sovereign.rs::CgapFrameView) match the frozen C
//! headers (include/cgap.h, include/cgap_c_data.h) in field order, offset,
//! size, and alignment — via a parsed-order runtime mirror AND a compile-time
//! static-assert pair. Anti-vacuity: a field reorder in a /tmp sandbox header
//! copy must flip BOTH the C-compile leg (cc freestanding, negative-array
//! offsetof pins) and the Rust parsed-order mirror leg. Courts mutate /tmp
//! copies only; the shared tree is never touched.
use std::path::{Path, PathBuf};
use std::process::Command;

use cgap_arrow::c_abi::{ArrowArray, ArrowSchema};
use cgap_ffi::sovereign::CgapFrameView;

/// One parsed C struct: name + field list in declaration order.
struct CStruct {
    name: String,
    fields: Vec<String>,
    decls: Vec<String>,
}

/// Strip /* */ and // comments from one line.
fn strip_comment(line: &str) -> &str {
    match line.find("/*") {
        Some(i) => &line[..i],
        None => match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        },
    }
}

/// Classify a C field declarator into a layout kind (LP64 host).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Ptr,
    I64,
    U32,
}

impl Kind {
    fn size(self) -> usize {
        match self {
            Kind::Ptr | Kind::I64 => 8,
            Kind::U32 => 4,
        }
    }
}

/// C declarator -> layout kind. Pointers (any `*`), int64_t, uint32_t.
fn kind_of(decl: &str) -> Option<Kind> {
    if decl.contains('*') {
        return Some(Kind::Ptr);
    }
    if decl.contains("int64_t") {
        return Some(Kind::I64);
    }
    if decl.contains("uint32_t") {
        return Some(Kind::U32);
    }
    None
}

/// Field name from a declarator: `(*name)` for function pointers, else the
/// last identifier token before the terminator.
fn field_name(decl: &str) -> String {
    if let Some(i) = decl.find("(*") {
        let rest = &decl[i + 2..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    let toks: Vec<&str> = decl
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .collect();
    toks.iter()
        .rev()
        .find(|t| !t.is_empty())
        .unwrap()
        .to_string()
}

/// Parse `struct NAME { ... };` and `typedef struct { ... } NAME;` bodies.
/// Returns (name, field-name list, field-declarator list) in file order.
fn parse_structs(src: &str) -> Vec<CStruct> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = strip_comment(lines[i]).trim();
        let (is_tagged, is_typedef, name_start) = if line.starts_with("typedef struct {") {
            (false, true, None)
        } else if line.starts_with("struct ") && line.contains("{") {
            (true, false, Some(line["struct ".len()..].trim()))
        } else {
            (false, false, None)
        };
        if !is_tagged && !is_typedef {
            i += 1;
            continue;
        }
        let name = if is_typedef {
            // name is on the closing `} NAME;` line
            let mut j = i;
            let mut name = String::new();
            while j < lines.len() {
                let l = strip_comment(lines[j]).trim();
                if l.starts_with('}') && l.contains(';') {
                    let seg = l.trim_start_matches('}').trim_end_matches(';').trim();
                    name = seg
                        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .find(|t| !t.is_empty())
                        .unwrap_or("")
                        .to_string();
                    break;
                }
                j += 1;
            }
            let _ = j;
            name
        } else {
            let seg = name_start.unwrap().trim_end_matches('{').trim();
            seg.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .find(|t| !t.is_empty())
                .unwrap_or("")
                .to_string()
        };
        // Walk to the body.
        let mut j = i;
        while j < lines.len() && !strip_comment(lines[j]).contains('{') {
            j += 1;
        }
        j += 1;
        let mut fields: Vec<String> = Vec::new();
        let mut decls: Vec<String> = Vec::new();
        while j < lines.len() {
            let raw = lines[j];
            let l = strip_comment(raw).trim();
            if l.starts_with('}') && l.contains(';') {
                break;
            }
            if !l.is_empty() && !l.starts_with('#') && !l.starts_with("_Static_assert") {
                // Accumulate until `;` (declarators may span lines).
                let mut acc = l.to_string();
                let mut k = j;
                while !acc.contains(';') && k + 1 < lines.len() {
                    k += 1;
                    acc.push(' ');
                    acc.push_str(strip_comment(lines[k]).trim());
                }
                let decl = acc.trim_end_matches(';').trim().to_string();
                if !decl.is_empty() {
                    fields.push(field_name(&decl));
                    decls.push(decl);
                }
                j = k;
            }
            j += 1;
        }
        out.push(CStruct {
            name,
            fields,
            decls,
        });
        i = j + 1;
    }
    out
}

/// Expected LP64 C layout for a field list, given per-field kinds.
/// Returns (field, offset) pairs plus total size and alignment.
fn c_layout(decls: &[(&str, Kind)]) -> (Vec<(String, usize)>, usize, usize) {
    let mut off = 0usize;
    let mut align = 1usize;
    let mut out = Vec::new();
    for (name, kind) in decls {
        let a = kind.size();
        off = (off + a - 1) & !(a - 1);
        out.push((name.to_string(), off));
        off += a;
        align = align.max(a);
    }
    let size = (off + align - 1) & !(align - 1);
    (out, size, align)
}

/// LP64 layout derived from the header's OWN parsed declarators — the
/// scoring path used by the mutation court. Unknown kinds fail loudly.
fn c_layout_from_decls(decls: &[String]) -> (Vec<(String, usize)>, usize, usize) {
    let mut off = 0usize;
    let mut align = 1usize;
    let mut out = Vec::new();
    for decl in decls {
        let kind = kind_of(decl).unwrap_or_else(|| panic!("unclassified C declarator: {}", decl));
        let a = kind.size();
        off = (off + a - 1) & !(a - 1);
        out.push((field_name(decl), off));
        off += a;
        align = align.max(a);
    }
    let size = (off + align - 1) & !(align - 1);
    (out, size, align)
}

/// The canonical field/kind inventory per struct (LP64), used both to score
/// the parsed C header and to derive expected offsets.
fn canonical(name: &str) -> Vec<(&'static str, Kind)> {
    match name {
        "cgap_frame_view" => vec![
            ("payload", Kind::Ptr),
            ("payload_len", Kind::I64),
            ("schema_id", Kind::U32),
            ("rule_count", Kind::U32),
        ],
        "ArrowArray" => vec![
            ("length", Kind::I64),
            ("null_count", Kind::I64),
            ("offset", Kind::I64),
            ("n_buffers", Kind::I64),
            ("n_children", Kind::I64),
            ("buffers", Kind::Ptr),
            ("children", Kind::Ptr),
            ("dictionary", Kind::Ptr),
            ("release", Kind::Ptr),
            ("private_data", Kind::Ptr),
        ],
        "ArrowSchema" => vec![
            ("format", Kind::Ptr),
            ("name", Kind::Ptr),
            ("metadata", Kind::Ptr),
            ("flags", Kind::I64),
            ("n_children", Kind::I64),
            ("children", Kind::Ptr),
            ("dictionary", Kind::Ptr),
            ("release", Kind::Ptr),
            ("private_data", Kind::Ptr),
        ],
        _ => Vec::new(),
    }
}

/// Rust-twin observed layout: (field, offset) in declaration order, plus
/// total size and alignment, read mechanically via offset_of!/size_of.
// the mirror's return shape tracks the parsed C header inventory 1:1
#[allow(clippy::type_complexity)]
fn rust_twin_layout(name: &str) -> Option<(Vec<(&'static str, usize)>, usize, usize)> {
    match name {
        "cgap_frame_view" => Some((
            vec![
                ("payload", core::mem::offset_of!(CgapFrameView, payload)),
                (
                    "payload_len",
                    core::mem::offset_of!(CgapFrameView, payload_len),
                ),
                ("schema_id", core::mem::offset_of!(CgapFrameView, schema_id)),
                (
                    "rule_count",
                    core::mem::offset_of!(CgapFrameView, rule_count),
                ),
            ],
            core::mem::size_of::<CgapFrameView>(),
            core::mem::align_of::<CgapFrameView>(),
        )),

        "ArrowArray" => Some((
            vec![
                ("length", core::mem::offset_of!(ArrowArray, length)),
                ("null_count", core::mem::offset_of!(ArrowArray, null_count)),
                ("offset", core::mem::offset_of!(ArrowArray, offset)),
                ("n_buffers", core::mem::offset_of!(ArrowArray, n_buffers)),
                ("n_children", core::mem::offset_of!(ArrowArray, n_children)),
                ("buffers", core::mem::offset_of!(ArrowArray, buffers)),
                ("children", core::mem::offset_of!(ArrowArray, children)),
                ("dictionary", core::mem::offset_of!(ArrowArray, dictionary)),
                ("release", core::mem::offset_of!(ArrowArray, release)),
                (
                    "private_data",
                    core::mem::offset_of!(ArrowArray, private_data),
                ),
            ],
            core::mem::size_of::<ArrowArray>(),
            core::mem::align_of::<ArrowArray>(),
        )),
        "ArrowSchema" => Some((
            vec![
                ("format", core::mem::offset_of!(ArrowSchema, format)),
                ("name", core::mem::offset_of!(ArrowSchema, name)),
                ("metadata", core::mem::offset_of!(ArrowSchema, metadata)),
                ("flags", core::mem::offset_of!(ArrowSchema, flags)),
                ("n_children", core::mem::offset_of!(ArrowSchema, n_children)),
                ("children", core::mem::offset_of!(ArrowSchema, children)),
                ("dictionary", core::mem::offset_of!(ArrowSchema, dictionary)),
                ("release", core::mem::offset_of!(ArrowSchema, release)),
                (
                    "private_data",
                    core::mem::offset_of!(ArrowSchema, private_data),
                ),
            ],
            core::mem::size_of::<ArrowSchema>(),
            core::mem::align_of::<ArrowSchema>(),
        )),
        _ => None,
    }
}

/// ---------------------------------------------------------------------------
/// Compile-time static-assert pair (Rust leg). If any twin's size, alignment,
/// or any field offset drifts, the whole test crate fails to COMPILE — the
/// same fail-closed posture as the header's own C11 _Static_assert pins.
const _: () = assert!(core::mem::size_of::<CgapFrameView>() == 24);
const _: () = assert!(core::mem::align_of::<CgapFrameView>() == 8);
const _: () = assert!(core::mem::offset_of!(CgapFrameView, payload) == 0);
const _: () = assert!(core::mem::offset_of!(CgapFrameView, payload_len) == 8);
const _: () = assert!(core::mem::offset_of!(CgapFrameView, schema_id) == 16);
const _: () = assert!(core::mem::offset_of!(CgapFrameView, rule_count) == 20);
const _: () = assert!(core::mem::size_of::<ArrowArray>() == 80);
const _: () = assert!(core::mem::align_of::<ArrowArray>() == 8);
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
const _: () = assert!(core::mem::size_of::<ArrowSchema>() == 72);
const _: () = assert!(core::mem::align_of::<ArrowSchema>() == 8);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, format) == 0);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, metadata) == 16);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, flags) == 24);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, n_children) == 32);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, children) == 40);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, dictionary) == 48);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, release) == 56);
const _: () = assert!(core::mem::offset_of!(ArrowSchema, private_data) == 64);

/// ---------------------------------------------------------------------------
/// Sandbox helpers: courts mutate /tmp copies only, never the shared tree.
fn repo_headers() -> (PathBuf, PathBuf) {
    (
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../include/cgap.h"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../include/cgap_c_data.h"),
    )
}

/// Fresh /tmp sandbox dir, unique per test process.
fn sandbox() -> PathBuf {
    let d = std::env::temp_dir().join(format!("cgap-e13-court-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("sandbox mkdir");
    d
}

/// Copy a header into the sandbox.
fn stage(sandbox_dir: &Path, src: &Path, name: &str) -> PathBuf {
    let dst = sandbox_dir.join(name);
    std::fs::copy(src, &dst).expect("stage header");
    dst
}

/// ANTI-VACUITY MUTATION: swap the first two field declarations of
/// `struct ArrowArray` (length <-> null_count) in a sandbox copy. A layout
/// court that still passes on this must be VACUOUS.
fn mutate_swap_first_two_fields(src: &str) -> String {
    let mut lines: Vec<String> = src.lines().map(String::from).collect();
    let start = lines
        .iter()
        .position(|l| l.trim() == "struct ArrowArray {")
        .expect("ArrowArray struct found for mutation");
    let mut i = start;
    while !lines[i].contains('{') {
        i += 1;
    }
    let field_idx: Vec<usize> = (start..lines.len())
        .take_while(|&k| !lines[k].trim_start().starts_with('}'))
        .filter(|&k| lines[k].trim().ends_with(';') && !lines[k].trim().starts_with('#'))
        .collect();
    assert!(field_idx.len() >= 2, "mutation needs >= 2 fields");
    let a = field_idx[0];
    let b = field_idx[1];
    lines.swap(a, b);
    lines.join("\n")
}

/// C99-compatible negative-array layout pins over offsetof/sizeof — the
/// freestanding compile leg (CL1 gate0 pattern). Any drift => compile error.
fn pins_c_source() -> String {
    let mut s =
        String::from("#include <stddef.h>\n#include \"cgap_c_data.h\"\n#include \"cgap.h\"\n");
    for (st, field, off) in [
        ("struct ArrowArray", "length", 0usize),
        ("struct ArrowArray", "null_count", 8),
        ("struct ArrowArray", "offset", 16),
        ("struct ArrowArray", "n_buffers", 24),
        ("struct ArrowArray", "n_children", 32),
        ("struct ArrowArray", "buffers", 40),
        ("struct ArrowArray", "children", 48),
        ("struct ArrowArray", "dictionary", 56),
        ("struct ArrowArray", "release", 64),
        ("struct ArrowArray", "private_data", 72),
    ] {
        s.push_str(&format!(
            "int pin_{f}_{o}[offsetof({st}, {f}) == {o} ? 1 : -1];\n",
            f = field,
            o = off,
            st = st
        ));
    }
    s.push_str("int pin_total[sizeof(struct ArrowArray) == 80 ? 1 : -1];\n");
    s
}

/// Compile a sandbox translation unit with cc, freestanding. Returns
/// (success, combined stderr+stdout).
fn cc_compile(dir: &PathBuf, c99: bool) -> (bool, String) {
    let out = dir.join("pin.o");
    let std_flag = if c99 { "-std=c99" } else { "-std=c11" };
    let res = Command::new("cc")
        .arg(std_flag)
        .arg("-ffreestanding")
        .arg("-I")
        .arg(dir)
        .arg("-c")
        .arg(dir.join("pins.c"))
        .arg("-o")
        .arg(&out)
        .output()
        .expect("spawn cc");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&res.stdout),
        String::from_utf8_lossy(&res.stderr)
    );
    (res.status.success(), log)
}

/// Copy BOTH headers into a sandbox dir, write pins.c, run cc.
fn compile_leg(dir: &PathBuf, c99: bool) -> (bool, String) {
    let (h1, h2) = repo_headers();
    let _ = stage(dir, &h1, "cgap.h");
    let _ = stage(dir, &h2, "cgap_c_data.h");
    std::fs::write(dir.join("pins.c"), pins_c_source()).expect("write pins.c");
    cc_compile(dir, c99)
}

// ---------------------------------------------------------------------------
// Courts. Every test names its subject and its falsifier.
// ---------------------------------------------------------------------------

/// COURT 1 — header inventory is parseable and matches the canonical
/// struct/field inventory (nothing added/removed/renamed in either header).
/// Falsifier: any header struct/field drift flips this via the parsed order.
#[test]
fn header_inventory_matches_canonical_structs() {
    let (h1, h2) = repo_headers();
    let cgap = std::fs::read_to_string(&h1).expect("read cgap.h");
    let cdata = std::fs::read_to_string(&h2).expect("read cgap_c_data.h");
    let mut parsed = parse_structs(&cgap);
    parsed.extend(parse_structs(&cdata));
    let got: Vec<(String, Vec<String>)> = parsed
        .iter()
        .map(|s| (s.name.clone(), s.fields.clone()))
        .collect();
    let want: Vec<(String, Vec<String>)> = ["cgap_frame_view", "ArrowArray", "ArrowSchema"]
        .iter()
        .map(|n| {
            (
                n.to_string(),
                canonical(n).iter().map(|(f, _)| f.to_string()).collect(),
            )
        })
        .collect();
    assert_eq!(got.len(), want.len(), "struct count drift: got {:?}", got);
    for (g, w) in got.iter().zip(want.iter()) {
        assert_eq!(g, w, "struct inventory drift");
    }
}

/// COURT 2 — Rust twins match the parsed C header: field order, per-field
/// offset, total size, alignment. This is the Rust-side mirror leg.
/// Falsifier: a header field reorder or a Rust twin edit flips this.
#[test]
fn rust_twins_match_parsed_header_layout() {
    let (h1, h2) = repo_headers();
    let cgap = std::fs::read_to_string(&h1).expect("read cgap.h");
    let cdata = std::fs::read_to_string(&h2).expect("read cgap_c_data.h");
    let mut parsed = parse_structs(&cgap);
    parsed.extend(parse_structs(&cdata));
    for s in &parsed {
        let (offs, size, align) = c_layout(&canonical(&s.name));
        let (offs_r, size_r, align_r) = rust_twin_layout(&s.name)
            .unwrap_or_else(|| panic!("no Rust twin for C struct {}", s.name));
        let names_c: Vec<&str> = offs.iter().map(|(f, _)| f.as_str()).collect();
        let names_r: Vec<&str> = offs_r.iter().map(|(f, _)| *f).collect();
        assert_eq!(names_c, names_r, "field order drift in {}", s.name);
        assert_eq!(
            offs,
            offs_r
                .iter()
                .map(|(f, o)| (f.to_string(), *o))
                .collect::<Vec<_>>(),
            "field offset drift in {}",
            s.name
        );
        assert_eq!(size, size_r, "size drift in {}", s.name);
        assert_eq!(align, align_r, "alignment drift in {}", s.name);
    }
}

/// COURT 3 — anti-vacuity: reorder a field in a /tmp header copy and the
/// C-compile leg MUST FAIL (baseline unmutated copy MUST compile clean).
/// The shared tree is never mutated; both flips are real cc invocations.
#[test]
fn c_compile_leg_flips_on_field_reorder() {
    let dir = sandbox();
    let (h1, h2) = repo_headers();
    let _ = stage(&dir, &h1, "cgap.h");
    let mutated =
        mutate_swap_first_two_fields(&std::fs::read_to_string(&h2).expect("read cgap_c_data.h"));
    std::fs::write(dir.join("cgap_c_data.h"), mutated).expect("write mutated header");
    std::fs::write(dir.join("pins.c"), pins_c_source()).expect("write pins.c");
    let (ok, log) = cc_compile(&dir, true);
    assert!(
        !ok,
        "VACUOUS COURT: pins.c compiled clean on a REORDERED ArrowArray header\n{}",
        log
    );
    // The failure must name the mutated subject, not some unrelated error.
    assert!(
        log.contains("pin_total") || log.contains("pin_null_count") || log.contains("offsetof"),
        "failure must cite a layout pin, got:\n{}",
        log
    );
    // Baseline: the pristine copy compiles clean — proves the flip came from
    // the mutation, not a broken leg.
    let dir2 = sandbox();
    let (ok0, log0) = compile_leg(&dir2, true);
    assert!(ok0, "baseline pristine header failed to compile:\n{}", log0);
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

/// COURT 4 — anti-vacuity, Rust mirror leg: the same mutated header copy,
/// scored through the parser + layout math over the header's OWN declarators,
/// must FAIL the twin check — while the pristine header scores EQUAL.
#[test]
fn rust_mirror_leg_flips_on_field_reorder() {
    let (h1, h2) = repo_headers();
    let pristine = std::fs::read_to_string(&h2).expect("read cgap_c_data.h");
    let mutated = mutate_swap_first_two_fields(&pristine);
    let _ = h1;
    let score = |src: &str| {
        let mut parsed = parse_structs(src);
        parsed.retain(|s| s.name == "ArrowArray");
        assert_eq!(parsed.len(), 1, "ArrowArray missing after parse");
        let (offs, size, _align) = c_layout_from_decls(&parsed[0].decls);
        let (offs_r, size_r, _align_r) = rust_twin_layout("ArrowArray").unwrap();
        offs.iter()
            .zip(offs_r.iter())
            .all(|((a, ao), (b, bo))| a == b && ao == bo)
            && offs.len() == offs_r.len()
            && size == size_r
    };
    assert!(
        score(&pristine),
        "pristine header must score EQUAL (court leg is wired, not vacuous)"
    );
    assert!(
        !score(&mutated),
        "VACUOUS COURT: Rust mirror accepted a REORDERED ArrowArray header"
    );
}
