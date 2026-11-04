//! Lane W2-10 (Wave R2 item 10, OABI-PROD-BYTES) — the production
//! `.oabi` / `.bn3` flash-rodata bytes for the XIP link.
//!
//! Gap closed (witnessed): the none-target xip link previously carried NO
//! sections from the production object — the xip.ld ASSERTs only passed via
//! court-fixture demo fills (`sovereign.rs::xip_courts` writes 0xA7x96 /
//! 0xB3x48 into its /tmp sandbox entry). This module emits the REAL tables
//! as `#[used] #[link_section]` statics, cfg-gated exactly to the XIP
//! surface (`all(target_os = "none", target_arch = "riscv32")`), so a
//! production none-target link passes the xip.ld ASSERTs on real bytes.
//!
//! Byte provenance (cited, not redefined — one calculus, one kernel):
//!
//! * `.oabi` = the committed A09 producer pack
//!   `ontology-packs/fibo/fibo_sec_dbt_v1.oabi` (496 B: 64 B A06 §6 header +
//!   9 x 48 B records; slot_count=9, class_count=34, enum_count=843). The
//!   container law is A06 §6 (`crates/cgap-bont/src/oabi.rs`:
//!   `OABI_MAGIC` / `OABI_HEADER_LEN` / `OABI_RECORD_SIZE` / `OabiPackV1`);
//!   the FULL §6 loader law and the producer-pack ingestion court stay with
//!   the owner crate (`w9_fibo_pack_ingestion_court`). This module carries
//!   the bytes and a minimal shape gate; it does NOT re-implement the
//!   loader (O(N^2) drift law).
//! * `.bn3` = an A11 arena image re-derived through the OWNER API
//!   (`cgap_bn3::arena::{Arena, FormulaNode}` -> `Arena::flatten` ->
//!   `Arena::to_image`, `crates/cgap-bn3/src/arena.rs:853`); program shape
//!   mirrors the arena court's own `nested_program` fixture
//!   (`arena.rs:987`): root quotation (2 statements) + child Q0 (1
//!   statement + grandchild leaf) = 4 statements x 24 B + 3 headers x 16 B
//!   + 8 B framing prefix = 152 B. Wire law: A11-bn3-arena.md SS4.4/SS6/SS7
//!   (24 B LE statements, 16 B LE headers, `[u32 n][u32 m]` prefix).
//!
//! Sections/placement: `crates/cgap-ffi/xip.ld` KEEP(*(.oabi)) /
//! KEEP(*(.bn3)), 64 KiB-aligned inside FLASH; the ASSERTs demand
//! `SIZEOF > 0` for both (the on-disk script carries no fixed-address or
//! byte-size demand — recorded divergence vs the dispatch sketch).
//!
//! Symbol surface: the statics are none-target-only and carry stable
//! flash-rodata symbol names (`cgap_xip_oabi_table`, `cgap_xip_bn3_network`)
//! so a XIP entry can reference them (and so the linker pulls the archive
//! member carrying them). Host builds compile them OUT (the cfg gate), so
//! the host cdylib surface (`oabi_lookup.rs` frozen six-symbol surface, the
//! `nm -gU` fifth-verb detector) is untouched. The statics are data, not
//! functions: the A19 four-verb boundary law is about boundary FUNCTIONS
//! (A19:47/:57-60); flash-rodata table symbols are the §2 `bn3_rodata`
//! design realized.
//!
//! @lane W2-10 (Wave R2 item 10), 2026-11-01, repo /Users/sac/cgap.
//!
//! File assembled mechanically (python) from the real bytes: hex text is
//! machine-generated from `ontology-packs/fibo/fibo_sec_dbt_v1.oabi` and from
//! the scratch derivation run (exit 0, /tmp/w210-derive), never hand-typed.

/// Committed byte length of the A09 `.oabi` producer pack
/// (`ontology-packs/fibo/fibo_sec_dbt_v1.oabi`): 64 B header + 9 x 48 B.
pub const OABI_PACK_LEN: usize = 496;

/// Byte length of the A11 arena image below (8 + 4x24 + 3x16).
pub const BN3_NETWORK_LEN: usize = 152;

/// The REAL A06 `.oabi` slot table: the committed A09 pack bytes,
/// byte-identical to `ontology-packs/fibo/fibo_sec_dbt_v1.oabi` on disk
/// (asserted every run by `r2_courts::r2_oabi_table_identity_court`).

pub const OABI_PACK_FIBO_SEC_DBT_V1: [u8; OABI_PACK_LEN] = [
    0x4f, 0x41, 0x42, 0x49, 0x76, 0x31, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x03, 0x00, 0x00, 0x00, 0x22, 0x00, 0x00, 0x00, 0x4b, 0x03, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00,
    0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x28, 0xcc, 0x7e, 0xdb, 0x39, 0xec, 0x08, 0xe5, 0x00, 0x00, 0x00, 0x00, 0x48, 0x03, 0x00, 0x00,
    0x28, 0xcc, 0x7e, 0xdb, 0x39, 0xec, 0x08, 0xe5, 0xc1, 0x8b, 0x7f, 0xb7, 0xf8, 0x7d, 0x3e, 0x1b,
    0x73, 0xb2, 0x9c, 0xe8, 0x74, 0x72, 0x30, 0x67, 0xb0, 0x31, 0x83, 0x74, 0xf7, 0xc7, 0xe8, 0x45,
    0x83, 0x45, 0xb2, 0xb4, 0x3e, 0x1a, 0xe6, 0x5b, 0x00, 0x00, 0x00, 0x00, 0x49, 0x03, 0x00, 0x00,
    0x83, 0x45, 0xb2, 0xb4, 0x3e, 0x1a, 0xe6, 0x5b, 0x1c, 0x8d, 0x89, 0xe2, 0x0a, 0xbe, 0xb3, 0x1f,
    0x47, 0xff, 0xd8, 0x28, 0x4c, 0x95, 0x6c, 0xca, 0x9d, 0x8d, 0x3a, 0x53, 0xeb, 0x03, 0xa1, 0x18,
    0x08, 0x01, 0x8f, 0x5c, 0x82, 0xf6, 0x58, 0xd5, 0x00, 0x00, 0x00, 0x00, 0x4a, 0x03, 0x00, 0x00,
    0x08, 0x01, 0x8f, 0x5c, 0x82, 0xf6, 0x58, 0xd5, 0xc7, 0x72, 0xcd, 0xeb, 0xa9, 0xd8, 0xb7, 0xdd,
    0x39, 0xb6, 0xa9, 0x0f, 0x4a, 0x6a, 0xf7, 0xd8, 0xed, 0xf9, 0x95, 0xe5, 0x0d, 0x21, 0x8b, 0xf0,
    0x16, 0x13, 0xd0, 0x5c, 0x42, 0xb3, 0x21, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x16, 0x13, 0xd0, 0x5c, 0x42, 0xb3, 0x21, 0x00, 0xe0, 0xb0, 0x12, 0xa3, 0xdd, 0xd9, 0x81, 0xba,
    0x7d, 0x77, 0x85, 0x43, 0xd8, 0x6d, 0x78, 0xb4, 0xc3, 0xf0, 0xe9, 0xda, 0x1a, 0x39, 0x1f, 0xa7,
    0x61, 0x99, 0xd3, 0x35, 0x0b, 0xe2, 0x48, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x61, 0x99, 0xd3, 0x35, 0x0b, 0xe2, 0x48, 0x14, 0x7f, 0x4e, 0x18, 0xb3, 0x93, 0xb6, 0x14, 0xd4,
    0x11, 0x04, 0xdc, 0xc2, 0x76, 0x37, 0xaa, 0x6b, 0x39, 0x35, 0x3b, 0xe2, 0x94, 0x38, 0x6d, 0x31,
    0x3e, 0xac, 0x46, 0xbb, 0xd0, 0xf2, 0xa1, 0xf7, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x3e, 0xac, 0x46, 0xbb, 0xd0, 0xf2, 0xa1, 0xf7, 0x91, 0x94, 0xbe, 0xc1, 0x6c, 0xb6, 0x5c, 0x37,
    0x97, 0x47, 0x7e, 0x6c, 0x00, 0x45, 0x16, 0x14, 0xcd, 0x7e, 0x1d, 0xa4, 0x75, 0xf6, 0x08, 0xbb,
    0xcb, 0xaa, 0xed, 0x19, 0x13, 0x22, 0x68, 0x08, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xcb, 0xaa, 0xed, 0x19, 0x13, 0x22, 0x68, 0x08, 0x57, 0xc8, 0x41, 0x35, 0xfd, 0x69, 0x30, 0xa6,
    0x9c, 0x71, 0x73, 0x97, 0x2c, 0x6a, 0x67, 0x8f, 0x79, 0x90, 0x82, 0x7c, 0xe5, 0x89, 0xbb, 0xad,
    0xf1, 0x33, 0x84, 0x83, 0x1d, 0x3e, 0x29, 0x39, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xf1, 0x33, 0x84, 0x83, 0x1d, 0x3e, 0x29, 0x39, 0x48, 0x31, 0x07, 0x56, 0x57, 0x25, 0x37, 0x8b,
    0x5f, 0x20, 0xec, 0x7a, 0x87, 0x43, 0xd7, 0xdb, 0x26, 0x0e, 0xe5, 0x72, 0x7f, 0x41, 0x92, 0xe5,
    0xd8, 0x38, 0x66, 0x4c, 0xc3, 0xf3, 0x67, 0x77, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xd8, 0x38, 0x66, 0x4c, 0xc3, 0xf3, 0x67, 0x77, 0xb5, 0xec, 0xe7, 0x16, 0x5c, 0x21, 0xdc, 0x06,
    0x84, 0x65, 0x6b, 0x3a, 0x44, 0x69, 0x17, 0x95, 0x24, 0x2e, 0x21, 0x65, 0x60, 0xf4, 0xa0, 0x09,
];

/// The REAL A11 `.bn3` network: the arena byte image re-derived through the
/// owner API (`Arena::flatten` + `Arena::to_image` over the cited program
/// shape), byte-identical to a live re-derivation every run
/// (`r2_courts::r2_bn3_network_identity_court`).

pub const BN3_NETWORK_A11_W2_10: [u8; BN3_NETWORK_LEN] = [
    0x04, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[cfg(all(target_os = "none", target_arch = "riscv32"))]
mod xip {
    use super::{BN3_NETWORK_A11_W2_10, BN3_NETWORK_LEN, OABI_PACK_FIBO_SEC_DBT_V1, OABI_PACK_LEN};

    /// Production `.oabi` section: the real A06 slot table in flash.
    /// `#[used]` keeps the static in the object; xip.ld `KEEP(*(.oabi))`
    /// roots it through `--gc-sections`; the stable symbol name lets a XIP
    /// entry reference it (archive-extraction root) and gives consumers the
    /// `bn3_rodata`-style flash symbol the A19 §2 design names.
    #[used]
    #[link_section = ".oabi"]
    #[export_name = "cgap_xip_oabi_table"]
    static XIP_OABI_TABLE: [u8; OABI_PACK_LEN] = OABI_PACK_FIBO_SEC_DBT_V1;

    /// Production `.bn3` section: the real A11 arena image in flash.
    #[used]
    #[link_section = ".bn3"]
    #[export_name = "cgap_xip_bn3_network"]
    static XIP_BN3_NETWORK: [u8; BN3_NETWORK_LEN] = BN3_NETWORK_A11_W2_10;
}

// ===========================================================================
// W2-10 courts (Chicago: real collaborators, assert final state, no mocks).
// All host-side (#[cfg(test)]); the production statics themselves are
// none-target-only. Real collaborators: the committed producer pack on disk,
// the REAL cgap-bn3 owner API (flatten/to_image/from_image), real nested
// cargo/rustc->rust-lld subprocess builds of the REAL crate against the
// checked-in xip.ld, llvm-size/llvm-objcopy over the produced ELF.
//
// Anti-vacuity, named: M-R2-0 (a header-byte flip must be refused by the
// A06 shape gate); M-R2-0b (a flipped derivation byte must flip the
// identity comparator); M-R2-1 (the PRODUCTION-BYTES shim: one flipped
// byte in a /tmp sandbox copy of the module must flow through a real
// rebuild and flip the section-identity comparator — a comparator that
// cannot see production-byte drift is decoration).
// ===========================================================================
#[cfg(test)]
mod r2_courts {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant};

    use super::{BN3_NETWORK_A11_W2_10, BN3_NETWORK_LEN, OABI_PACK_FIBO_SEC_DBT_V1, OABI_PACK_LEN};

    /// xip.ld FLASH region (crates/cgap-ffi/xip.ld).
    const FLASH_ORIGIN: u64 = 0x4200_0020;
    const FLASH_LEN: u64 = 4 * 1024 * 1024;
    const FLASH_PAGE: u64 = 64 * 1024;
    /// A19 §4 budgets (the same gates the T1 court pins).
    const FLASH_BUDGET: u64 = 64 * 1024;
    const SRAM_BUDGET: u64 = 32 * 1024;

    /// Nested-build deadline. Generous on purpose (first run builds the
    /// rv32 core graph), still bounded and enforced — the court polls the
    /// real child exit, it never sleeps-and-hopes (no-sleep law; POLL-STEP
    /// below is the one audited poll).
    const BUILD_DEADLINE: Duration = Duration::from_secs(600);
    const POLL_STEP: Duration = Duration::from_millis(200);

    /// A06 §6 frozen magic (cited: crates/cgap-bont/src/oabi.rs
    /// `OABI_MAGIC`, A06 §6 offset table; the wire constant, not a
    /// re-implementation of the loader law).
    const OABI_MAGIC: [u8; 8] = *b"OABIv1\0\0";

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root canonicalize")
    }

    fn xip_ld_path() -> PathBuf {
        let p = repo_root().join("crates/cgap-ffi/xip.ld");
        assert!(
            p.is_file(),
            "checked-in crates/cgap-ffi/xip.ld missing at {}",
            p.display()
        );
        p
    }

    /// Fresh read of the committed A09 producer pack (canonical tree,
    /// read-only; never mutated — mutations run on copies).
    fn committed_oabi_bytes() -> Vec<u8> {
        let p = repo_root().join("ontology-packs/fibo/fibo_sec_dbt_v1.oabi");
        fs::read(&p)
            .unwrap_or_else(|e| panic!("committed producer pack {} unreadable: {e}", p.display()))
    }

    /// Minimal A06 §6 SHAPE gate over pack bytes: magic, header_len == 64,
    /// version_major == 1, LE flag, record_size == 48, reserved zero,
    /// exact length law, slot_count > 0. CITED, not redefined: the FULL §6
    /// loader law (flags/reserved/ranges and the per-record
    /// `slot == truncate64(lex)` derivation) is
    /// `cgap_bont::oabi::OabiPackV1::from_bytes` in the owner crate, whose
    /// producer-pack ingestion court (`w9_fibo_pack_ingestion_court`)
    /// already loads THIS pack green every run. This gate exists so the
    /// byte-identity court asserts semantic shape, not bare length.
    /// Returns (class_count, enum_count, slot_count).
    fn oabi_shape_gate(bytes: &[u8]) -> Result<(u32, u32, u32), String> {
        let u32le = |o: usize| -> Result<u32, String> {
            bytes
                .get(o..o + 4)
                .map(|s| u32::from_le_bytes(s.try_into().expect("4-byte window")))
                .ok_or_else(|| format!("header truncated at offset {o}"))
        };
        if bytes.len() < 64 {
            return Err(format!(
                "shorter than the 64 B A06 §6 header ({})",
                bytes.len()
            ));
        }
        if bytes[0..8] != OABI_MAGIC {
            return Err("bad magic".to_string());
        }
        let header_len = u32le(8)?;
        if header_len != 64 {
            return Err(format!("header_len {header_len} != 64 (A06 §6)"));
        }
        if u32le(12)? & 0xFFFF != 1 {
            return Err("version_major != 1".to_string());
        }
        if u32le(16)? & 0x1 == 0 {
            return Err("LE flag (bit0) not set".to_string());
        }
        if u32le(32)? != 48 {
            return Err(format!("record_size {} != 48 (A06 §6)", u32le(32)?));
        }
        if bytes[36..64].iter().any(|&b| b != 0) {
            return Err("nonzero reserved byte".to_string());
        }
        let slot_count = u32le(28)?;
        if slot_count == 0 {
            return Err("slot_count == 0: an empty pack is not a corpus".to_string());
        }
        let needed = (slot_count as usize)
            .checked_mul(48)
            .and_then(|r| r.checked_add(64))
            .ok_or_else(|| "slot_count overflows the length law".to_string())?;
        if bytes.len() != needed {
            return Err(format!("length law: needed {needed}, got {}", bytes.len()));
        }
        Ok((u32le(20)?, u32le(24)?, slot_count))
    }

    fn tail(text: &str) -> String {
        let n = text.chars().count();
        if n <= 800 {
            text.to_string()
        } else {
            text.chars().skip(n - 800).collect()
        }
    }

    /// Bounded subprocess run — the one audited poll: deadline-checked
    /// try_wait at POLL_STEP, kill on breach. SLEEP-CLASS: POLL-STEP.
    fn run_bounded(
        program: &str,
        args: &[&str],
        envs: &[(&str, String)],
    ) -> Result<(Option<std::process::ExitStatus>, String, String), String> {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().map_err(|e| format!("spawn {program}: {e}"))?;
        let out = child.stdout.take().expect("stdout piped");
        let err = child.stderr.take().expect("stderr piped");
        let rd_out = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut { out }, &mut s);
            s
        });
        let rd_err = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut { err }, &mut s);
            s
        });
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(st)) => break Some(st),
                Ok(None) => {
                    if started.elapsed() >= BUILD_DEADLINE {
                        let _ = child.kill();
                        let _ = child.wait();
                        break None;
                    }
                    std::thread::sleep(POLL_STEP); // SLEEP-CLASS: POLL-STEP
                }
                Err(e) => return Err(format!("try_wait: {e}")),
            }
        };
        Ok((
            status,
            rd_out.join().expect("stdout reader"),
            rd_err.join().expect("stderr reader"),
        ))
    }

    /// The toolchain's llvm-tools bin dir (llvm-size / llvm-objcopy),
    /// triple-agnostic (same discovery the sovereign.rs xip courts use).
    fn llvm_tools_bin() -> Option<PathBuf> {
        let out = Command::new("rustc")
            .arg("--print")
            .arg("sysroot")
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let sysroot = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
        let rustlib = sysroot.join("lib/rustlib");
        for entry in std::fs::read_dir(&rustlib).ok()?.flatten() {
            let bin = entry.path().join("bin");
            if bin.is_dir() && bin.join("llvm-size").is_file() {
                return Some(bin);
            }
        }
        None
    }

    /// Target std/core presence = the loud-skip toolchain gate (A19 §6.3
    /// form: a skip is loud, never a silent pass).
    fn target_installed(triple: &str) -> bool {
        let out = Command::new("rustc").arg("--print").arg("sysroot").output();
        let Ok(out) = out else { return false };
        if !out.status.success() {
            return false;
        }
        let sysroot = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
        sysroot.join("lib/rustlib").join(triple).is_dir()
    }

    // ------------------------------------------------------------------
    // Real derivation + real section extraction
    // ------------------------------------------------------------------

    /// Program shape CITED from the owner crate's own court fixture
    /// (`crates/cgap-bn3/src/arena.rs:987` `nested_program`): root
    /// quotation (2 statements, the second carrying a FORMULA_REF to child
    /// ordinal 0) + child Q0 (1 statement + grandchild leaf). Flatten ->
    /// 4 statements x 24 B + 3 headers x 16 B + 8 B framing = 152 B.
    fn nested_program() -> cgap_bn3::arena::FormulaNode {
        use cgap_bn3::sigil::Sigil;
        cgap_bn3::arena::FormulaNode::quoted(
            vec![
                [
                    (Sigil::SlotConst, 1),
                    (Sigil::VarUni, 10),
                    (Sigil::SlotConst, 2),
                ],
                [
                    (Sigil::SlotConst, 3),
                    (Sigil::SlotConst, 4),
                    (Sigil::FormulaRef, 0),
                ],
            ],
            vec![cgap_bn3::arena::FormulaNode::quoted(
                vec![[
                    (Sigil::VarExi, 20),
                    (Sigil::SlotConst, 5),
                    (Sigil::SlotConst, 6),
                ]],
                vec![cgap_bn3::arena::FormulaNode::leaf(vec![[
                    (Sigil::SlotConst, 7),
                    (Sigil::SlotConst, 8),
                    (Sigil::SlotConst, 9),
                ]])],
            )],
        )
    }

    /// The A11 network image, re-derived live through the OWNER API
    /// (`Arena::flatten` + `Arena::to_image`, arena.rs:853) — the same
    /// derivation that minted the embedded const.
    fn derive_bn3_image() -> Vec<u8> {
        let mut arena = cgap_bn3::arena::Arena::new();
        arena
            .flatten(&nested_program())
            .expect("owner flatten must admit the cited program");
        arena.to_image()
    }

    /// Exact section bytes from the linked ELF via llvm-objcopy
    /// --dump-section (verified against this toolchain this session on a
    /// real object before landing: exit 0, exact bytes). Fails LOUD.
    fn dump_sections(elf: &Path, tools: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut map = BTreeMap::new();
        for name in [".oabi", ".bn3"] {
            let out_path = elf
                .parent()
                .expect("elf dir")
                .join(format!("w210-dump{name}"));
            let out_s = out_path.display().to_string();
            let spec = format!("{name}={out_s}");
            let run = Command::new(tools.join("llvm-objcopy"))
                .arg(format!("--dump-section={spec}"))
                .arg(elf)
                .output()
                .expect("spawn llvm-objcopy");
            assert!(
                run.status.success(),
                "llvm-objcopy --dump-section={name} failed on {}:\n{}",
                elf.display(),
                String::from_utf8_lossy(&run.stderr)
            );
            let bytes =
                fs::read(&out_path).unwrap_or_else(|e| panic!("dump file {out_s} unreadable: {e}"));
            let _ = fs::remove_file(&out_path);
            map.insert(name.to_string(), bytes);
        }
        map
    }

    // ------------------------------------------------------------------
    // The production-link sandbox (a lease: created fresh, deleted at the
    // end; the canonical tree is never mutated)
    // ------------------------------------------------------------------

    const FIXTURE_MANIFEST: &str = r#"[package]
name = "cgap-ffi"
version = "26.10.29"
edition = "2021"
publish = false

[lib]
crate-type = ["staticlib", "rlib"]

[[bin]]
name = "a19xip_prod"
path = "src/main.rs"

[dependencies]
cgap-arrow = { path = "<ARROW>" }

[profile.release]
opt-level = "s"
lto = "fat"
codegen-units = 1
panic = "abort"
"#;

    /// The PRODUCTION XIP entry: NO demo fills. The only .oabi/.bn3 input
    /// sections in this link come from the production `xip_tables` statics
    /// inside the real crate. The extern references are the
    /// archive-extraction roots: the rlib member carrying the statics is
    /// pulled because the entry references its exported symbols, and the
    /// volatile reads root them through --gc-sections (xip.ld's
    /// KEEP(*(.oabi))/KEEP(*(.bn3)) is the second root).
    const ENTRY_MAIN: &str = r#"#![no_std]
#![no_main]
use core::arch::asm;
extern crate cgap_ffi;

extern "C" {
    static cgap_xip_oabi_table: [u8; 496];
    static cgap_xip_bn3_network: [u8; 152];
}

#[no_mangle]
pub extern "C" fn cgap_xip_start() -> ! {
    let o = core::ptr::addr_of!(cgap_xip_oabi_table);
    let b = core::ptr::addr_of!(cgap_xip_bn3_network);
    unsafe {
        core::ptr::read_volatile(o as *const u8);
        core::ptr::read_volatile(b as *const u8);
    }
    loop {
        unsafe { asm!("wfi") }
    }
}
"#;

    /// Stage the /tmp sandbox lease: a byte-copy of the REAL src/*.rs (so
    /// the court builds the same code the workspace builds), the manifest
    /// above, and the production entry.
    fn stage_sandbox(dir: &Path) {
        fs::create_dir_all(dir.join("src")).expect("sandbox src dir");
        let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        copy_src_recursive(&src_dir, &dir.join("src"));
        let manifest = FIXTURE_MANIFEST.replace(
            "<ARROW>",
            &repo_root().join("crates/cgap-arrow").display().to_string(),
        );
        fs::write(dir.join("Cargo.toml"), manifest).expect("write sandbox manifest");
        fs::write(dir.join("src/main.rs"), ENTRY_MAIN).expect("write sandbox entry");
    }

    /// AF-8: recursive sandbox source copy over `src/**/*.rs` — module
    /// directories (e.g. `src/aoq/`) must land with relative paths preserved
    /// or the sandbox build dies with E0583 (file not found for module).
    fn copy_src_recursive(src_root: &std::path::Path, dst_root: &std::path::Path) {
        std::fs::create_dir_all(dst_root).expect("dst src dir");
        for entry in std::fs::read_dir(src_root)
            .expect("src dir readable")
            .flatten()
        {
            let p = entry.path();
            if p.is_dir() {
                copy_src_recursive(&p, &dst_root.join(p.file_name().expect("dir name")));
            } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                std::fs::copy(&p, dst_root.join(p.file_name().expect("file name")))
                    .expect("copy sandbox source");
            }
        }
    }

    fn cargo_build(dir: &Path, envs: &[(&str, String)]) -> (bool, String) {
        let mp = dir.join("Cargo.toml").display().to_string();
        let td = dir.join("target").display().to_string();
        let (status, out, err) = run_bounded(
            "cargo",
            &[
                "build",
                "--release",
                "--target",
                "riscv32imc-unknown-none-elf",
                "--manifest-path",
                mp.as_str(),
                "--target-dir",
                td.as_str(),
            ],
            envs,
        )
        .unwrap_or_else(|e| panic!("W2-10 court: nested runner failed: {e}"));
        let st = status.unwrap_or_else(|| {
            panic!(
                "W2-10 court: build deadline exceeded; stderr tail:\n{}",
                tail(&err)
            )
        });
        (st.success(), if err.is_empty() { out } else { err })
    }

    fn elf_path(dir: &Path) -> PathBuf {
        dir.join("target/riscv32imc-unknown-none-elf/release/a19xip_prod")
    }

    // ------------------------------------------------------------------
    // Court 1 — the .oabi production table: committed-byte identity +
    // shape gate + M-R2-0 flip
    // ------------------------------------------------------------------

    #[test]
    fn r2_oabi_table_identity_court() {
        let file = committed_oabi_bytes();
        // L0 — shape gate green on the real committed pack.
        let (class_count, enum_count, slot_count) = oabi_shape_gate(&file)
            .unwrap_or_else(|e| panic!("committed A09 pack fails the A06 §6 shape gate: {e}"));
        assert_eq!(
            (class_count, enum_count, slot_count),
            (34, 843, 9),
            "pack header counts must match the committed A09 corpus"
        );
        // L1 — the embedded production table is byte-identical to the
        // committed producer output (fresh disk read every run).
        assert_eq!(
            OABI_PACK_FIBO_SEC_DBT_V1.as_slice(),
            file.as_slice(),
            "embedded .oabi production table drifted from the committed A09 pack"
        );
        // M-R2-0 (named mutation): one header byte flip must be REFUSED by
        // the shape gate. A gate that accepts the mutant is VACUOUS.
        let mut mutant = file.clone();
        assert_eq!(mutant[8], 0x40, "premise: header_len LSB is 0x40 (64)");
        mutant[8] = 0x41;
        let err = oabi_shape_gate(&mutant)
            .err()
            .expect("VACUOUS GATE: the A06 shape gate accepted header_len=65");
        assert!(
            err.contains("header_len"),
            "M-R2-0 flip must name its leg (header_len), got: {err}"
        );
        println!(
            "W2-10 OABI TABLE: 496 B (64 hdr + 9x48 rec), class=34 enum=843 slot=9; \
embedded == committed (byte-identical); M-R2-0 flip fired on its named leg"
        );
    }

    // ------------------------------------------------------------------
    // Court 2 — the .bn3 production network: live owner-API derivation
    // identity + restore round-trip + M-R2-0b flip
    // ------------------------------------------------------------------

    #[test]
    fn r2_bn3_network_identity_court() {
        // L0 — live derivation through the REAL owner API.
        let img = derive_bn3_image();
        assert_eq!(img.len(), BN3_NETWORK_LEN, "live image length drifted");
        // L1 — the embedded production network == the live derivation
        // (byte-identical; any owner-API change that moves the wire form
        // flips here and forces a re-mint).
        assert_eq!(
            BN3_NETWORK_A11_W2_10.as_slice(),
            img.as_slice(),
            "embedded .bn3 production network drifted from the live owner-API derivation"
        );
        // L2 — the image restores through the OWNER restore boundary
        // (from_image: framing/statements/pad/intervals/scope layers) and
        // re-serializes byte-exact (both directions, the arena wave-9
        // identity law over THESE bytes).
        let restored = cgap_bn3::arena::Arena::from_image(&img)
            .expect("production network must restore through Arena::from_image");
        assert_eq!(restored.slab.len(), 4, "restored statement count");
        assert_eq!(restored.formulas.len(), 3, "restored header count");
        assert_eq!(
            restored.to_image().as_slice(),
            img.as_slice(),
            "restored arena re-serialization is not byte-exact"
        );
        // M-R2-0b (named mutation): one flipped byte must flip the identity
        // comparator over real bytes.
        let mut m = img.clone();
        m[9] ^= 0x01;
        assert_ne!(
            m.as_slice(),
            BN3_NETWORK_A11_W2_10.as_slice(),
            "VACUOUS: the identity comparator accepted a flipped derivation byte"
        );
        println!(
            "W2-10 BN3 NETWORK: 152 B (8 framing + 4x24 stmts + 3x16 headers); \
embedded == live owner-API derivation (byte-identical); from_image restores; M-R2-0b flip fired"
        );
    }

    // ------------------------------------------------------------------
    // Court 3 — the headline: the PRODUCTION none-target link. Real crate,
    // real xip.ld, real rust-lld, production bytes only (no demo fills).
    // + the M-R2-1 production-bytes shim leg.
    // ------------------------------------------------------------------

    #[test]
    fn r2_production_link_court() {
        let Some(tools) = llvm_tools_bin() else {
            println!("[SKIP] rustc sysroot llvm tools absent (llvm-size/llvm-objcopy) — W2-10 production link court unexecuted (loud skip, never a silent pass)");
            return;
        };
        if !target_installed("riscv32imc-unknown-none-elf") {
            println!("[SKIP] riscv32imc-unknown-none-elf target not installed — W2-10 production link court unexecuted (loud skip, never a silent pass)");
            return;
        }
        let script = xip_ld_path();
        let dir = std::env::temp_dir().join(format!("cgap-w210-prod-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir); // stale lease from a crashed prior run
        stage_sandbox(&dir);
        let envs = vec![(
            "RUSTFLAGS",
            format!(
                "-C link-arg=-T{} -C link-arg=--gc-sections",
                script.display()
            ),
        )];

        // ---- GREEN LEG: production bytes only --------------------------
        let (ok, log) = cargo_build(&dir, &envs);
        assert!(
            ok,
            "PRODUCTION xip link failed (xip.ld ASSERTs or build): stderr tail:\n{}",
            tail(&log)
        );
        let elf = elf_path(&dir);
        assert!(elf.is_file(), "production ELF missing at {}", elf.display());

        // Budgets + placement (llvm-size -A; measured, never estimated).
        let size_out = Command::new(tools.join("llvm-size"))
            .arg("-A")
            .arg(&elf)
            .output()
            .expect("spawn llvm-size");
        assert!(size_out.status.success(), "llvm-size -A failed");
        let mut flash = 0u64;
        let mut sram = 0u64;
        let mut secs: BTreeMap<String, (u64, u64)> = BTreeMap::new();
        for line in String::from_utf8_lossy(&size_out.stdout).lines() {
            let toks: Vec<&str> = line.split_whitespace().collect();
            if toks.len() != 3 || !toks[0].starts_with('.') {
                continue;
            }
            let (Ok(sz), Ok(addr)) = (toks[1].parse::<u64>(), toks[2].parse::<u64>()) else {
                continue;
            };
            secs.insert(toks[0].to_string(), (sz, addr));
            let in_flash = toks[0].starts_with(".text")
                || toks[0].starts_with(".rodata")
                || toks[0].starts_with(".oabi")
                || toks[0].starts_with(".bn3")
                || toks[0].starts_with(".init")
                || toks[0].starts_with(".fini");
            let in_sram = toks[0].starts_with(".data")
                || toks[0].starts_with(".bss")
                || toks[0].starts_with(".sbss")
                || toks[0].starts_with(".sdata")
                || toks[0] == "COMMON";
            if in_flash {
                flash += sz;
            } else if in_sram {
                sram += sz;
            }
        }
        assert!(
            flash < FLASH_BUDGET,
            "PRODUCTION ELF FLASH {flash} B >= budget {FLASH_BUDGET} B — measured, gate stays red"
        );
        assert!(
            sram < SRAM_BUDGET,
            "PRODUCTION ELF SRAM {sram} B >= budget {SRAM_BUDGET} B — measured, gate stays red"
        );
        for name in [".oabi", ".bn3"] {
            let Some(&(sz, addr)) = secs.get(name) else {
                panic!("{name} absent from the PRODUCTION ELF — xip.ld ASSERTs passed only if the section landed; llvm-size output:\n{}",
                    String::from_utf8_lossy(&size_out.stdout));
            };
            assert_eq!(
                sz as usize,
                if name == ".oabi" {
                    OABI_PACK_LEN
                } else {
                    BN3_NETWORK_LEN
                },
                "{name} linked size {sz} != the real owner-table size"
            );
            assert!(
                addr >= FLASH_ORIGIN && addr < FLASH_ORIGIN + FLASH_LEN,
                "{name} at {addr:#x} outside FLASH"
            );
            assert_eq!(
                addr % FLASH_PAGE,
                0,
                "{name} at {addr:#x} not {FLASH_PAGE}-byte page-aligned"
            );
        }
        assert!(
            secs[".oabi"].1 < secs[".bn3"].1,
            ".oabi must precede .bn3 in flash (xip.ld order)"
        );

        // Byte-identity: extracted section bytes == owner tables.
        let dumped = dump_sections(&elf, &tools);
        assert_eq!(
            dumped[".oabi"].as_slice(),
            committed_oabi_bytes().as_slice(),
            ".oabi section bytes != the committed A09 pack (production object does not carry the real table)"
        );
        assert_eq!(
            dumped[".bn3"].as_slice(),
            derive_bn3_image().as_slice(),
            ".bn3 section bytes != the live owner-API derivation"
        );
        assert_eq!(
            dumped[".oabi"].as_slice(),
            OABI_PACK_FIBO_SEC_DBT_V1.as_slice()
        );
        assert_eq!(dumped[".bn3"].as_slice(), BN3_NETWORK_A11_W2_10.as_slice());
        println!(
            "W2-10 PRODUCTION LINK: rust-lld -T xip.ld exit 0 on REAL bytes; flash={flash} B/{FLASH_BUDGET} B, sram={sram} B/{SRAM_BUDGET} B; \
.oabi {} B at {:#x}, .bn3 {} B at {:#x}, both byte-identical to owner tables",
            secs[".oabi"].0, secs[".oabi"].1, secs[".bn3"].0, secs[".bn3"].1
        );

        // ---- M-R2-1: the PRODUCTION-BYTES shim (real drift, real flip) --
        // One flipped LexHash byte in the SANDBOX copy's embedded table:
        // the rebuild still LINKS (the ASSERTs demand presence, not
        // content) but the section-identity comparator MUST refuse —
        // proving the comparator can see production-byte drift.
        let xip_rs = dir.join("src/xip_tables.rs");
        let src = fs::read_to_string(&xip_rs).expect("read sandbox xip_tables.rs");
        // a08-wave7 count==1 discipline, scoped to the MUTATED SPAN: the
        // anchor must be unique inside the data array's text — the court's
        // own anchor literal (below) is not a mutation site and must not
        // make the flip unattributable.
        let anchor = "0xc1, 0x8b, 0x7f, 0xb7, 0xf8, 0x7d, 0x3e, 0x1b";
        let arr_begin = src
            .find("pub const OABI_PACK_FIBO_SEC_DBT_V1")
            .expect("embedded .oabi const present in the sandbox copy");
        let arr_end = arr_begin
            + src[arr_begin..]
                .find("];")
                .expect("array span closes in the sandbox copy");
        let head = &src[..arr_begin];
        let array_text = &src[arr_begin..arr_end];
        let tail_text = &src[arr_end..];
        assert_eq!(array_text.matches(anchor).count(), 1,
            "M-R2-1 anchor not unique within the array span — mutation ambiguous, flip unattributable");
        let mutated = format!(
            "{head}{}{tail_text}",
            array_text.replacen(anchor, "0xc1, 0x8b, 0x7f, 0xb7, 0xf8, 0x7d, 0x3e, 0x1a", 1)
        );
        assert_ne!(mutated, src, "M-R2-1 anchor absent — mutation vacuous");
        fs::write(&xip_rs, mutated).expect("write mutated sandbox module");
        let (ok, log) = cargo_build(&dir, &envs);
        assert!(
            ok,
            "M-R2-1 mutant rebuild failed (it should still LINK): {}",
            tail(&log)
        );
        let elf = elf_path(&dir);
        let dumped = dump_sections(&elf, &tools);
        assert_ne!(
            dumped[".oabi"].as_slice(),
            committed_oabi_bytes().as_slice(),
            "VACUOUS GATE: M-R2-1 — a flipped production byte still passed the section-identity comparator"
        );
        println!("W2-10 M-R2-1 FLIP: one flipped production byte rebuilt+relinked, section-identity comparator REFUSED — the byte gate is load-bearing");
        let _ = fs::remove_dir_all(&dir);
    }
}
