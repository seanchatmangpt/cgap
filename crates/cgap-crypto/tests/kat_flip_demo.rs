//! AF-12 — live negative-mutation flip demonstration court.
//!
//! Promise under test: flipping one bit in a cryptographic round constant
//! forces an immediate automated rejection. This court makes it executable:
//! copy the landed kernel source to a /tmp SANDBOX (never the repo tree),
//! flip one bit/byte of the constant IN THE COPY, compile the mutated kernel
//! standalone with rustc, execute it, and require the KAT / roundtrip court
//! to reject — for every single flip — plus a meta-court proving the demo
//! is not vacuous.
//!
//! Families (baselines bound to the LANDED kernel's public APIs in-court):
//!   A. ChaCha SIGMA "expand 32-byte k": all 128 single-bit flips, one at a
//!      time; each must change >= 1 byte of the RFC 8439 sec 2.3.2 block
//!      fixture (exhaustive avalanche witness).
//!   B. fips202 round-constant table RC[0..24]: each of the 192 bytes
//!      flipped; each must make the FIPS 202 SHAKE-256("") KAT reject.
//!   C. fips203 ZETAS: low bit of entries [1, 2, 64, 127] (all consumed by
//!      ntt()/intt_tomont()); each must fail the ntt∘intt_tomont roundtrip
//!      self-test AND change the NTT output.
//!   M. Meta-court: flips_detected == flips_total, the unflipped baseline
//!      passes the same KATs against the LANDED kernel (not only the
//!      sandbox), and the repo tree is restored byte-exact.
//!
//! Zero external dependencies (std only). A court that accepts its own
//! mutation is a VACUOUS GATE and exits 1.
//!
//! Run: cargo test -p cgap-crypto --test kat_flip_demo -- --nocapture
//!
//! Sandbox pattern: repo's ct_scan_court.rs (AB-4) — /tmp copies, never the
//! repo tree, fail-closed restore witness.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::thread;

const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");

const RFC8439_232_BLOCK: &str = "10f1e7e4d13b5915500fdd1fa32071c4\
     c7d1f4c733c068030422aa9ac3d46c4e\
     d2826446079faa0914c2d705d98b02a2\
     b5129cd1de164eb9cbd083e8a2503c4e";
const SHAKE256_EMPTY_64: &str = "46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762f\
     d75dc4ddd8c0f200cb05019d67b592f6fc821c49479ab48640292eacb3b7c4be";

/// Kernel files under mutation (paths under the crate's src/).
const KERNELS: [&str; 3] = ["chacha.rs", "fips202.rs", "fips203.rs"];

// ---------------------------------------------------------------------------
// Sandbox build+run machinery (mutate a /tmp copy of the kernel source,
// never the repo tree; fail-closed restore witness).
// ---------------------------------------------------------------------------

struct Sandbox {
    dir: PathBuf,
}

impl Sandbox {
    fn new(tag: &str) -> Sandbox {
        Sandbox {
            dir: std::env::temp_dir().join(format!("af12-flip-{}-{tag}", std::process::id())),
        }
    }

    /// Write kernel files + driver, compile with rustc, run, return stdout.
    fn build_and_run(&self, files: &[(&str, &str)], driver: &str) -> Result<String, String> {
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        for (name, text) in files {
            fs::write(self.dir.join(name), text).map_err(|e| e.to_string())?;
        }
        fs::write(self.dir.join("main.rs"), driver).map_err(|e| e.to_string())?;
        let bin = self.dir.join("bin");
        let out = Command::new("rustc")
            .arg("--edition=2021")
            .arg("-Awarnings")
            .arg("-o")
            .arg(&bin)
            .arg(self.dir.join("main.rs"))
            .output()
            .map_err(|e| format!("rustc spawn failed (is rustc on PATH?): {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "sandbox rustc failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        let run = Command::new(&bin)
            .output()
            .map_err(|e| format!("sandbox bin spawn failed: {e}"))?;
        let _ = fs::remove_dir_all(&self.dir);
        if !run.status.success() {
            return Err(format!(
                "sandbox bin failed: {}",
                String::from_utf8_lossy(&run.stderr)
            ));
        }
        String::from_utf8(run.stdout).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Drivers: standalone mains compiled against the sandbox kernel copies.
// ---------------------------------------------------------------------------

const CHACHA_DRIVER: &str = r#"#[path = "chacha.rs"] mod chacha;
fn main() {
    let key: [u8; 32] = core::array::from_fn(|i| i as u8);
    let nonce: [u8; 12] =
        [0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];
    let out = chacha::chacha20_block(&key, 1, &nonce);
    for b in out {
        print!("{:02x}", b);
    }
    println!();
}
"#;

const FIPS202_DRIVER: &str = r#"#[path = "fips202.rs"] mod fips202;
fn main() {
    let mut out = [0u8; 64];
    fips202::Shake256::xof(&[], &mut out);
    for b in out {
        print!("{:02x}", b);
    }
    println!();
}
"#;

const FIPS203_DRIVER: &str = r#"#[path = "fips202.rs"] mod fips202;
#[path = "fips203.rs"] mod fips203;
fn main() {
    let mut p = [0i16; 256];
    for i in 0..256 {
        p[i] = ((i * 7 + 3) % 3329) as i16;
    }
    let p0 = p;
    fips203::ntt(&mut p);
    for c in p {
        print!("{:04x}", c as u16);
    }
    println!();
    // NTT self-test: intt_tomont(ntt(p)) == p scaled by 2^16 mod q, which is
    // fqmul(x, 1) (Montgomery division by R). A flipped zeta breaks the
    // transform-pair identity -> self-test rejects.
    let mut q = p0;
    fips203::ntt(&mut q);
    fips203::intt_tomont(&mut q);
    let ok = (0..256).all(|i| {
        let norm = |x: i16| ((x as i32).rem_euclid(3329)) as i16;
        norm(fips203::fqmul(q[i], 1)) == norm(p0[i])
    });
    println!("SELFTEST {}", if ok { "PASS" } else { "FAIL" });
}
"#;

// ---------------------------------------------------------------------------
// Constant-block mutation helpers.
// ---------------------------------------------------------------------------

/// Map the numeric-literal tokens (decimal, optional '-'; or 0x-hex with '_')
/// inside a bracketed const block: token k is rebuilt by `map(k, value)`.
/// Non-token text (whitespace, commas) is copied through verbatim.
fn map_const_tokens<F>(block: &str, mut map: F) -> String
where
    F: FnMut(usize, i64) -> Option<String>,
{
    let b = block.as_bytes();
    let mut out = String::with_capacity(block.len() + 64);
    let mut i = 0;
    let mut k = 0;
    while i < b.len() {
        let neg = b[i] == b'-' && i + 1 < b.len() && b[i + 1].is_ascii_digit();
        let hex = b[i] == b'0' && i + 1 < b.len() && b[i + 1] == b'x';
        if b[i].is_ascii_digit() || neg {
            let start = i;
            let mut j = i + if neg { 1 } else { 0 };
            if hex {
                j += 2;
            }
            while j < b.len()
                && (if hex {
                    b[j].is_ascii_hexdigit() || b[j] == b'_'
                } else {
                    b[j].is_ascii_digit()
                })
            {
                j += 1;
            }
            let text = &block[start..j];
            let digits = if hex { &text[2..] } else { text };
            let clean = digits.replace('_', "");
            let v = i128::from_str_radix(&clean, if hex { 16 } else { 10 }).expect("token parse");
            let v = if neg { -v } else { v };
            let v = v as i64;
            match map(k, v) {
                Some(repl) => out.push_str(&repl),
                None => out.push_str(text),
            }
            k += 1;
            i = j;
        } else {
            out.push_str(&block[i..i + 1]);
            i += 1;
        }
    }
    out
}

/// Extract the `[ ... ];` body of the const block that follows `header`.
fn const_block(src: &str, header: &str) -> (usize, usize) {
    let h = src
        .find(header)
        .unwrap_or_else(|| panic!("const block header not found: {header}"));
    let eq = h + src[h..].find("= [").expect("no '= [' after header");
    let open = eq + 2;
    let close = open + src[open..].find("];").expect("unterminated const block") + 1;
    (open, close)
}

/// Flip bit 0 (low byte flip) of SIGMA word `word`.
fn flip_sigma(src: &str, word: usize, bit: u32) -> String {
    const WORDS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];
    let mut flipped = WORDS;
    flipped[word] ^= 1 << bit;
    let old_lit = format!(
        "[0x{:04x}_{:04x}, 0x{:04x}_{:04x}, 0x{:04x}_{:04x}, 0x{:04x}_{:04x}]",
        WORDS[0] >> 16,
        WORDS[0] & 0xffff,
        WORDS[1] >> 16,
        WORDS[1] & 0xffff,
        WORDS[2] >> 16,
        WORDS[2] & 0xffff,
        WORDS[3] >> 16,
        WORDS[3] & 0xffff
    );
    let new_lit = format!(
        "[0x{:08X}, 0x{:08X}, 0x{:08X}, 0x{:08X}]",
        flipped[0], flipped[1], flipped[2], flipped[3]
    );
    assert!(
        src.contains(&old_lit),
        "SIGMA literal not found in chacha.rs — kernel drifted; court refuses to run blind"
    );
    src.replace(&old_lit, &new_lit)
}

// ---------------------------------------------------------------------------
// Flip families -> sandbox outcomes.
// ---------------------------------------------------------------------------

struct Row {
    family: &'static str,
    label: String,
    detected: bool,
    changed_bytes: usize,
    detail: String,
}

struct Report {
    rows: Vec<Row>,
    baselines_ok: bool,
    restore_ok: bool,
    baseline_detail: String,
}

fn strip_ws(hex: &str) -> String {
    hex.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Hex-difference count between two equal-length hex strings.
fn diff_bytes(a: &str, b: &str) -> usize {
    let da = hex_decode(a.trim());
    let db = hex_decode(b.trim());
    if da.len() != db.len() {
        return usize::MAX;
    }
    da.iter().zip(db.iter()).filter(|(x, y)| x != y).count()
}

/// Decode a hex string to bytes; len must be even, else empty (signals a
/// malformed sandbox output and forces diff_bytes to report a hard failure).
fn hex_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    if b.len() % 2 != 0 || !b.iter().all(|c| c.is_ascii_hexdigit()) {
        return Vec::new();
    }
    (0..b.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex byte"))
        .collect()
}

fn kernel_text(name: &str) -> String {
    let p = PathBuf::from(MANIFEST).join("src").join(name);
    String::from_utf8_lossy(&fs::read(&p).expect("read kernel source")).into_owned()
}

/// One flip: build + run the mutated sandbox, compare against expectations.
fn run_flip(
    family: &'static str,
    label: String,
    files: Vec<(&'static str, String)>,
    driver: &str,
    baseline_out: &str,
    want_reject: bool,
) -> Row {
    let tag = format!("{}-{}", family, label.replace([' ', '[', ']', ','], "_"));
    let sbx = Sandbox::new(&tag);
    let refs: Vec<(&str, &str)> = files.iter().map(|(n, t)| (*n, t.as_str())).collect();
    match sbx.build_and_run(&refs, driver) {
        Ok(out) => {
            let out = out.trim();
            if family == "zetas" {
                // Driver prints NTT hex then SELFTEST PASS/FAIL.
                let (hex, verdict) = match out.rsplit_once('\n') {
                    Some((h, v)) => (h.trim(), v.trim()),
                    None => (out, ""),
                };
                let changed = diff_bytes(hex, baseline_out);
                let rejects = verdict.ends_with("FAIL") && changed > 0;
                Row {
                    family,
                    label,
                    detected: rejects == want_reject,
                    changed_bytes: changed,
                    detail: format!("selftest={verdict} changed={changed}"),
                }
            } else {
                let changed = diff_bytes(out, baseline_out);
                Row {
                    family,
                    label,
                    detected: (changed > 0) == want_reject,
                    changed_bytes: changed,
                    detail: format!("changed={changed}"),
                }
            }
        }
        Err(e) => Row {
            family,
            label,
            detected: false,
            changed_bytes: 0,
            detail: format!("SANDBOX ERROR: {e}"),
        },
    }
}

fn run_all() -> Report {
    let before: Vec<(String, Vec<u8>)> = KERNELS
        .iter()
        .map(|k| {
            (
                k.to_string(),
                fs::read(PathBuf::from(MANIFEST).join("src").join(k)).expect("read"),
            )
        })
        .collect();

    // ---- Baselines: sandbox must reproduce the LANDED kernel's outputs,
    // ---- and the landed kernel must pass its own KATs in-court.
    let mut baseline_detail = String::new();
    let mut baselines_ok = true;

    let chacha_src = kernel_text("chacha.rs");
    let fips202_src = kernel_text("fips202.rs");
    let fips203_src = kernel_text("fips203.rs");

    let kat_key: [u8; 32] = core::array::from_fn(|i| i as u8);
    let kat_nonce: [u8; 12] = [
        0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
    ];
    let landed_block = cgap_crypto::chacha::chacha20_block(&kat_key, 1, &kat_nonce);
    let landed_hex = landed_block
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let rfc = strip_ws(RFC8439_232_BLOCK);
    if landed_hex != rfc {
        baselines_ok = false;
        baseline_detail.push_str("LANDED chacha20_block != RFC 8439 2.3.2 fixture; ");
    }

    let mut shk = [0u8; 64];
    cgap_crypto::fips202::Shake256::xof(&[], &mut shk);
    let landed_shk = shk.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let official = strip_ws(SHAKE256_EMPTY_64);
    if landed_shk != official {
        baselines_ok = false;
        baseline_detail.push_str("LANDED Shake256(\"\") != FIPS 202 KAT; ");
    }

    let mut p0 = [0i16; 256];
    for (i, c) in p0.iter_mut().enumerate() {
        *c = ((i * 7 + 3) % 3329) as i16;
    }
    let mut q = p0;
    cgap_crypto::fips203::ntt(&mut q);
    cgap_crypto::fips203::intt_tomont(&mut q);
    let canon = |x: i16| ((x as i32).rem_euclid(3329)) as i16;
    let roundtrip_ok =
        (0..256).all(|i| canon(cgap_crypto::fips203::fqmul(q[i], 1)) == canon(p0[i]));
    if !roundtrip_ok {
        baselines_ok = false;
        baseline_detail.push_str("LANDED ntt/intt_tomont roundtrip fails; ");
    }

    // Sandbox baselines (unflipped copies) must match the landed outputs.
    let sbx_block =
        Sandbox::new("base-chacha").build_and_run(&[("chacha.rs", &chacha_src)], CHACHA_DRIVER);
    let sbx_shake =
        Sandbox::new("base-f202").build_and_run(&[("fips202.rs", &fips202_src)], FIPS202_DRIVER);
    let sbx_ntt = Sandbox::new("base-f203").build_and_run(
        &[("fips202.rs", &fips202_src), ("fips203.rs", &fips203_src)],
        FIPS203_DRIVER,
    );
    let (chacha_base, shake_base, ntt_base_and_verdict) = match (sbx_block, sbx_shake, sbx_ntt) {
        (Ok(a), Ok(b), Ok(c)) => (
            a.trim().to_string(),
            b.trim().to_string(),
            c.trim().to_string(),
        ),
        (e, f, g) => {
            baselines_ok = false;
            let msg = format!(
                "SANDBOX BASELINE ERRORS: chacha={:?} f202={:?} f203={:?}",
                e.err(),
                f.err(),
                g.err()
            );
            baseline_detail.push_str(&msg);
            eprintln!("{msg}");
            return finish(before, Vec::new(), baselines_ok, baseline_detail);
        }
    };
    if chacha_base != landed_hex || shake_base != landed_shk {
        baselines_ok = false;
        baseline_detail.push_str("sandbox baseline != landed kernel output; ");
    }
    let ntt_base = match ntt_base_and_verdict.rsplit_once('\n') {
        Some((h, v)) if v.ends_with("PASS") => h.trim().to_string(),
        _ => {
            baselines_ok = false;
            baseline_detail.push_str("sandbox NTT baseline selftest failed; ");
            String::new()
        }
    };
    println!(
        "AF-12 baselines: chacha RFC-8439-fixture={} shake256-KAT={} ntt-roundtrip={}",
        chacha_base == landed_hex,
        shake_base == landed_shk,
        roundtrip_ok
    );

    // ---- Build the flip cases.
type FlipCase<'a> = (
    &'static str,
    String,
    Vec<(&'static str, String)>,
    &'static str,
    &'a str,
    bool,
);
    let mut cases: Vec<FlipCase<'_>> = Vec::new();
    for word in 0..4usize {
        for bit in 0..32u32 {
            cases.push((
                "sigma",
                format!("sigma w{word} b{bit:02}"),
                vec![("chacha.rs", flip_sigma(&chacha_src, word, bit))],
                CHACHA_DRIVER,
                &chacha_base,
                true,
            ));
        }
    }
    for idx in 0..192usize {
        cases.push((
            "rc",
            format!("rc byte{idx:03}"),
            vec![("fips202.rs", flip_rc_byte(&fips202_src, idx))],
            FIPS202_DRIVER,
            &shake_base,
            true,
        ));
    }
    for entry in [1usize, 2, 64, 127] {
        cases.push((
            "zetas",
            format!("zetas[{entry}] ^1"),
            vec![
                ("fips202.rs", fips202_src.clone()),
                ("fips203.rs", flip_zetas_entry(&fips203_src, entry)),
            ],
            FIPS203_DRIVER,
            &ntt_base,
            true,
        ));
    }

    // ---- Execute flips, thread-limited.
    let n_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8);
    let rows: Vec<Row> = thread::scope(|s| {
        let mut handles = Vec::new();
        let mut chunks: Vec<Vec<_>> = Vec::new();
        let chunk = cases.len().div_ceil(n_threads);
        for c in cases.chunks(chunk) {
            chunks.push(c.to_vec());
        }
        for mut part in chunks {
            handles.push(s.spawn(move || {
                let mut out = Vec::new();
                for (fam, label, files, drv, base, want) in part.drain(..) {
                    out.push(run_flip(fam, label, files, drv, base, want));
                }
                out
            }));
        }
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("flip worker panicked"))
            .collect()
    });

    finish(before, rows, baselines_ok, baseline_detail)
}

fn finish(
    before: Vec<(String, Vec<u8>)>,
    rows: Vec<Row>,
    baselines_ok: bool,
    baseline_detail: String,
) -> Report {
    let restore_ok = before.iter().all(|(k, v)| {
        *v == fs::read(PathBuf::from(MANIFEST).join("src").join(k)).expect("re-read")
    });
    let report = Report {
        rows,
        baselines_ok,
        restore_ok,
        baseline_detail,
    };
    print_tables(&report);
    report
}

fn print_tables(r: &Report) {
    for family in ["sigma", "rc", "zetas"] {
        println!("\n== AF-12 family: {family} ==");
        println!("{:<22} {:>9} {:>8}  detail", "flip", "detected", "changed");
        for row in r.rows.iter().filter(|row| row.family == family) {
            println!(
                "{:<22} {:>9} {:>8}  {}",
                row.label,
                if row.detected { "PASS" } else { "FAIL" },
                row.changed_bytes,
                row.detail
            );
        }
    }
    let total = r.rows.len();
    let detected = r.rows.iter().filter(|row| row.detected).count();
    println!("\n== AF-12 meta-court ==");
    println!(
        "flips_total={total} flips_detected={detected} baselines_ok={} restore_ok={}",
        r.baselines_ok, r.restore_ok
    );
}

/// Flip byte `idx` of RC (idx = 8*round + byte, little-endian byte order
/// within the u64): token k = RC[idx / 8], byte = idx % 8.
fn flip_rc_byte(src: &str, idx: usize) -> String {
    let (open, close) = const_block(src, "const RC: [u64; 24] = [");
    let block = &src[open..=close];
    let mut seen = 0usize;
    let mutated = map_const_tokens(block, |k, v| {
        if k == idx / 8 {
            seen += 1;
            let byte = idx % 8;
            let nv = ((v as u64) ^ (1u64 << (8 * byte))) as i64;
            Some(format!("0x{nv:016X}"))
        } else {
            None
        }
    });
    let _ = seen;
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..open]);
    out.push_str(&mutated);
    out.push_str(&src[close + 1..]);
    out
}

/// Flip the low bit of ZETAS[`entry`].
fn flip_zetas_entry(src: &str, entry: usize) -> String {
    let (open, close) = const_block(src, "pub const ZETAS: [i16; 259] = [");
    let block = &src[open..=close];
    let mutated = map_const_tokens(block, |k, v| {
        if k == entry {
            Some(format!("{}", (v as i16) ^ 1))
        } else {
            None
        }
    });
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..open]);
    out.push_str(&mutated);
    out.push_str(&src[close + 1..]);
    out
}

// ---------------------------------------------------------------------------
// Courts. The report is computed once; each test asserts its slice.
// ---------------------------------------------------------------------------

static REPORT: OnceLock<Report> = OnceLock::new();

fn report() -> &'static Report {
    REPORT.get_or_init(run_all)
}

fn assert_slice(family: &'static str) {
    let r = report();
    let rows: Vec<&Row> = r.rows.iter().filter(|row| row.family == family).collect();
    assert!(!rows.is_empty(), "family {family} produced no flips");
    for row in &rows {
        assert!(
            row.detected,
            "UNDETECTED MUTATION [{}] {}: {} — a flip the court accepted is a vacuous gate",
            row.family, row.label, row.detail
        );
    }
}

/// Court A: every single-bit flip of the ChaCha SIGMA constant must change
/// the RFC 8439 sec 2.3.2 block output (exhaustive avalanche witness).
#[test]
fn sigma_bit_flips_all_change_block_output() {
    assert_slice("sigma");
    let r = report();
    let sigma = r.rows.iter().filter(|row| row.family == "sigma").count();
    assert_eq!(sigma, 128, "SIGMA is 4 u32 words = 128 bits, one flip each");
}

/// Court B: every byte flip of the fips202 RC table must make the
/// SHAKE-256("") KAT reject.
#[test]
fn rc_byte_flips_all_rejected_by_kat() {
    assert_slice("rc");
    let r = report();
    let rc = r.rows.iter().filter(|row| row.family == "rc").count();
    assert_eq!(rc, 192, "RC is 24 u64 = 192 bytes, one flip each");
}

/// Court C: a flipped ZETAS entry must fail the NTT roundtrip self-test.
#[test]
fn zetas_flip_rejected_by_ntt_selftest() {
    assert_slice("zetas");
}

/// Court M (meta): the demo is not vacuous — every flip detected, unflipped
/// baselines pass (landed kernel + sandbox), repo tree restored byte-exact.
#[test]
fn meta_court_flip_demo_is_not_vacuous() {
    let r = report();
    let total = r.rows.len();
    let detected = r.rows.iter().filter(|row| row.detected).count();
    assert_eq!(detected, total, "flips_detected must equal flips_total");
    assert_eq!(total, 128 + 192 + 4, "SIGMA 128 + RC 192 + ZETAS 4 = 324");
    assert!(
        r.baselines_ok,
        "unflipped baseline must pass: {}",
        r.baseline_detail
    );
    assert!(r.restore_ok, "repo tree must be restored byte-exact");
}
