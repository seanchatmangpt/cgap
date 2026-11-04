//! cbmqs_runner — AF-14 CBMQS bake-off runner (harness=false gate).
//!
//! Executes the 7-gate CBMQS standard against a subject:
//!   G1 alloc-per-op census      G2 timing census
//!   G3 mock-framework scan      G4 census replay determinism
//!   G5 storm feed               G6 kill-cycles (process subjects)
//!   G7 report integrity (BLAKE3 hash + runner version stamp)
//!
//! Modes:
//!   (no args)      self-test: subject = cgap-aoq itself, in-process. The
//!                  runner must PASS its own standard — the anti-vacuity
//!                  proof that the standard is satisfiable.
//!   --court        non-vacuity court: self-test + /tmp fixture subjects
//!                  that violate G1/G5, plus mock-tree refusal and a
//!                  report-tamper court. Every gate must flip somewhere.
//!   --subject-cmd <shell command>   foreign subprocess subject speaking
//!                  the frame protocol (see docs/cbmqs-runner.md).
//!
//! Exit codes: 0 verdict PASS · 2 verdict FAIL · 3 verdict REFUSED ·
//! 1 usage/infrastructure error. The exit status IS the verdict (repo
//! exit-record law).
//!
//! Verdict reports are JSON, content-hashed (BLAKE3 via cgap-blake3) and
//! stamped with the runner's own version so verdicts are reproducible.
//! Timing numbers are measured; census counts are deterministic.

use std::alloc::{GlobalAlloc, System};
use std::io::{BufRead, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

const RUNNER_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+af14.1");
const STANDARD: &str = "CBMQS-7G/1";

// ---------------------------------------------------------------------------
// Counting allocator: the G1/G4 census instrument.
// ---------------------------------------------------------------------------

struct CensusAlloc;

static CENSUS_ALLOCS: AtomicU64 = AtomicU64::new(0);
static CENSUS_BYTES: AtomicU64 = AtomicU64::new(0);
static CENSUS_LIVE: AtomicU64 = AtomicU64::new(0);
static CENSUS_ENABLED: AtomicBool = AtomicBool::new(false);

unsafe impl GlobalAlloc for CensusAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() && CENSUS_ENABLED.load(Ordering::Relaxed) {
            CENSUS_ALLOCS.fetch_add(1, Ordering::Relaxed);
            CENSUS_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, p: *mut u8, layout: std::alloc::Layout) {
        if CENSUS_ENABLED.load(Ordering::Relaxed) {
            CENSUS_LIVE.fetch_sub(1, Ordering::Relaxed);
        }
        System.dealloc(p, layout)
    }
}

#[global_allocator]
static GLOBAL: CensusAlloc = CensusAlloc;

/// Run `f` inside a census window; returns (allocs, bytes).
fn census_window<F: FnOnce()>(f: F) -> (u64, u64) {
    CENSUS_ALLOCS.store(0, Ordering::Relaxed);
    CENSUS_BYTES.store(0, Ordering::Relaxed);
    CENSUS_ENABLED.store(true, Ordering::Relaxed);
    f();
    CENSUS_ENABLED.store(false, Ordering::Relaxed);
    (
        CENSUS_ALLOCS.load(Ordering::Relaxed),
        CENSUS_BYTES.load(Ordering::Relaxed),
    )
}

// ---------------------------------------------------------------------------
// Report model (hand-rolled JSON; no serde on an embedded surface).
// ---------------------------------------------------------------------------

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// One gate outcome. `status` ∈ PASS | FAIL | SKIP; `kind` is the typed
/// reason code (e.g. PASS, OVERALLOC, CONTRACT_UNSUPPORTED, CRASH,
/// SUBJECT_IS_LIBRARY); `measured` carries the real numbers.
#[derive(Clone)]
struct GateResult {
    gate: &'static str,
    status: &'static str,
    kind: String,
    measured: String,
    note: String,
}

impl GateResult {
    fn pass(gate: &'static str, kind: &str, measured: &str, note: &str) -> Self {
        Self {
            gate,
            status: "PASS",
            kind: kind.into(),
            measured: measured.into(),
            note: note.into(),
        }
    }
    fn fail(gate: &'static str, kind: &str, measured: &str, note: &str) -> Self {
        Self {
            gate,
            status: "FAIL",
            kind: kind.into(),
            measured: measured.into(),
            note: note.into(),
        }
    }
    fn skip(gate: &'static str, kind: &str, measured: &str, note: &str) -> Self {
        Self {
            gate,
            status: "SKIP",
            kind: kind.into(),
            measured: measured.into(),
            note: note.into(),
        }
    }
}

struct Report {
    subject_mode: &'static str,
    subject_id: String,
    gates: Vec<GateResult>,
    alloc_bound_per_op: u64,
    time_budget_ns: u64,
    gates_requested: String,
    verdict: &'static str,
    refusal: Option<String>,
    blake3_hex: String,
}

impl Report {
    /// Canonical body: the full report with an EMPTY hash field. The G7
    /// hash is BLAKE3 over exactly these bytes.
    fn canonical_body(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"runner\": \"cbmqs-runner\",\n"));
        s.push_str(&format!(
            "  \"runner_version\": \"{}\",\n",
            json_escape(RUNNER_VERSION)
        ));
        s.push_str(&format!("  \"standard\": \"{}\",\n", json_escape(STANDARD)));
        s.push_str(&format!("  \"subject_mode\": \"{}\",\n", self.subject_mode));
        s.push_str(&format!(
            "  \"subject_id\": \"{}\",\n",
            json_escape(&self.subject_id)
        ));
        s.push_str(&format!(
            "  \"alloc_bound_per_op\": {},\n  \"time_budget_ns\": {},\n",
            self.alloc_bound_per_op, self.time_budget_ns
        ));
        s.push_str(&format!(
            "  \"gates_requested\": \"{}\",\n",
            json_escape(&self.gates_requested)
        ));
        for g in &self.gates {
            s.push_str(&format!(
                "  {{ \"gate\": \"{}\", \"status\": \"{}\", \"kind\": \"{}\", \"measured\": \"{}\", \"note\": \"{}\" }},\n",
                g.gate, g.status, json_escape(&g.kind), json_escape(&g.measured), json_escape(&g.note)
            ));
        }
        if let Some(r) = &self.refusal {
            s.push_str(&format!("  \"refusal\": \"{}\",\n", json_escape(r)));
        }
        s.push_str(&format!("  \"verdict\": \"{}\",\n", self.verdict));
        s.push_str("  \"report_hash_algo\": \"blake3-32\",\n");
        s.push_str("  \"report_hash\": \"\"\n");
        s.push_str("}");
        s
    }

    /// Seal: compute the BLAKE3 hash over the canonical body and inject it.
    fn seal(mut self) -> Self {
        let body = self.canonical_body();
        let h = cgap_blake3::hash(body.as_bytes());
        let hex: String = h.iter().map(|b| format!("{:02x}", b)).collect();
        self.blake3_hex = hex;
        self
    }

    fn render(&self) -> String {
        let body = self.canonical_body();
        body.replace(
            "\"report_hash\": \"\"",
            &format!("\"report_hash\": \"{}\"", self.blake3_hex),
        )
    }

    /// G7 verification law: strip the printed hash, re-canonicalize,
    /// re-hash; the printed hash must equal the recomputed one.
    fn verify_hash_law(report_json: &str) -> Result<(), String> {
        const MARK: &str = "\"report_hash\": \"";
        let start = report_json.find(MARK).ok_or("no report_hash field")? + MARK.len();
        let end = report_json[start..]
            .find('"')
            .ok_or("unterminated report_hash")?
            + start;
        let printed = &report_json[start..end];
        let body = report_json.replace(
            &format!("\"report_hash\": \"{}\"", printed),
            "\"report_hash\": \"\"",
        );
        let recomputed = cgap_blake3::hash(body.as_bytes());
        let hex: String = recomputed.iter().map(|b| format!("{:02x}", b)).collect();
        if hex == printed {
            Ok(())
        } else {
            Err(format!(
                "hash mismatch: printed {} recomputed {}",
                printed, hex
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Self-test subject: cgap-aoq itself, in-process.
// ---------------------------------------------------------------------------

use cgap_aoq::codec::{decode, encode};
use cgap_aoq::error::AoqError;
use cgap_aoq::header::{AoqHeader, Epoch, MAGIC, VERSION};

const CENSUS_OPS: usize = 8192;

fn genesis_header(frame_ordinal: u32) -> AoqHeader {
    AoqHeader {
        magic: MAGIC,
        version: VERSION,
        epoch: Epoch::Genesis as u16,
        key_id: 0x0102_0304_0506_0708,
        frame_ordinal,
        seq: frame_ordinal.wrapping_mul(7),
        reserved: [0u8; 36],
        header_digest: 0,
    }
}

/// The subject's hot-path op: one full encode+decode roundtrip of a real
/// 64-byte AOQ1 envelope. Alloc-free by the codec's own law (bound 0).
fn aoq_op(i: usize) {
    let mut header = genesis_header(i as u32);
    let mut wire = [0u8; 64];
    encode(&header, &mut wire).expect("self-test encode");
    let back = decode(&wire).expect("self-test decode");
    std::hint::black_box(&back);
}

fn alloc_positive_control() -> u64 {
    // Anti-vacuity: an op that MUST allocate. If the census instrument
    // cannot see a real allocation, every G1 verdict is vacuous.
    let (allocs, _) = census_window(|| {
        let v: Vec<u8> = Vec::with_capacity(4096);
        std::hint::black_box(&v);
    });
    allocs
}

fn gate_g1_self() -> GateResult {
    let control = alloc_positive_control();
    if control == 0 {
        return GateResult::fail("G1", "VACUOUS_ALLOC_CENSUS", "control_allocs=0",
            "counting allocator saw no allocation in a definitely-allocating window; census instrument is dead");
    }
    // Warmup outside the window (lazy init, page-in).
    aoq_op(0);
    let (allocs, bytes) = census_window(|| {
        for i in 0..CENSUS_OPS {
            aoq_op(i);
        }
    });
    let per_op = allocs / CENSUS_OPS as u64;
    if per_op == 0 && allocs == 0 {
        GateResult::pass("G1", "PASS",
            &format!("ops={} allocs=0 bytes=0 control_allocs={}", CENSUS_OPS, control),
            "alloc-free codec law holds (bound 0 allocs/op); positive control witnessed the instrument live")
    } else {
        GateResult::fail(
            "G1",
            "OVERALLOC",
            &format!(
                "ops={} allocs={} bytes={} per_op={}",
                CENSUS_OPS, allocs, bytes, per_op
            ),
            "codec allocated inside the hot-path window",
        )
    }
}

fn percentile_p99(sorted: &[u32]) -> u32 {
    let idx = ((sorted.len() as u64 * 99) / 100).max(1) as usize - 1;
    sorted[idx.min(sorted.len() - 1)]
}

fn gate_g2_self() -> GateResult {
    const BUDGET_NS: u64 = 10_000;
    aoq_op(0); // warmup
    let mut per_op_ns: Vec<u32> = Vec::with_capacity(CENSUS_OPS);
    for i in 0..CENSUS_OPS {
        let t0 = Instant::now();
        aoq_op(i);
        per_op_ns.push(t0.elapsed().subsec_nanos());
    }
    per_op_ns.sort_unstable();
    let p50 = per_op_ns[per_op_ns.len() / 2];
    let p99 = percentile_p99(&per_op_ns);
    if (p99 as u64) <= BUDGET_NS {
        GateResult::pass(
            "G2",
            "PASS",
            &format!(
                "ops={} p50_ns={} p99_ns={} budget_ns={}",
                CENSUS_OPS, p50, p99, BUDGET_NS
            ),
            "timing census within the disclosed budget (budget is a TARGET, not a truth claim)",
        )
    } else {
        GateResult::fail(
            "G2",
            "TIME_BUDGET_EXCEEDED",
            &format!(
                "ops={} p50_ns={} p99_ns={} budget_ns={}",
                CENSUS_OPS, p50, p99, BUDGET_NS
            ),
            "p99 per-op latency exceeded the disclosed budget",
        )
    }
}

fn gate_g4_self() -> GateResult {
    aoq_op(0);
    let a = census_window(|| {
        for i in 0..CENSUS_OPS {
            aoq_op(i);
        }
    });
    let b = census_window(|| {
        for i in 0..CENSUS_OPS {
            aoq_op(i);
        }
    });
    if a.0 == b.0 && a.1 == b.1 && a.0 == 0 {
        GateResult::pass(
            "G4",
            "PASS",
            &format!("run1=({}a,{}B) run2=({}a,{}B)", a.0, a.1, b.0, b.1),
            "two census windows byte-identical (timing is measured, not required replayable)",
        )
    } else {
        GateResult::fail(
            "G4",
            "CENSUS_REPLAY_DIVERGED",
            &format!("run1=({}a,{}B) run2=({}a,{}B)", a.0, a.1, b.0, b.1),
            "census windows diverged across replay",
        )
    }
}

// ---------------------------------------------------------------------------
// G3 mock-framework scan (anti-rigging) + G5 self storm.
// ---------------------------------------------------------------------------

const MOCK_PATTERNS: &[&str] = &[
    "mockall",
    "mock!",
    "gmock",
    "jest.mock",
    "vi.mock",
    "unittest.mock",
    "monkeypatch",
    "sinon.stub",
    "mock(",
    "double::",
    "faux::create",
];
const SCAN_SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "__pycache__", ".venv"];

fn scan_tree(root: &str) -> Result<Vec<String>, String> {
    let mut hits = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(root)];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                if !SCAN_SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(p);
                }
            } else if p
                .extension()
                .map(|x| x.to_string_lossy().len() <= 8)
                .unwrap_or(false)
            {
                if let Ok(content) = std::fs::read(&p) {
                    if content.len() < 4 * 1024 * 1024 {
                        let hay = String::from_utf8_lossy(&content).to_lowercase();
                        for pat in MOCK_PATTERNS {
                            if hay.contains(pat) {
                                hits.push(format!("{}: contains \"{}\"", p.display(), pat));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(hits)
}

fn gate_g3_scan(source_tree: &str, self_mode: bool) -> GateResult {
    match scan_tree(source_tree) {
        Ok(hits) if hits.is_empty() => GateResult::pass(
            "G3",
            "PASS",
            &format!(
                "tree={} files_scanned=ok patterns={}",
                source_tree,
                MOCK_PATTERNS.len()
            ),
            "no mock-framework patterns in the subject source tree",
        ),
        Ok(hits) => GateResult::fail(
            "G3",
            "MOCK_DETECTED",
            &hits.join("; "),
            "mock framework referenced in subject source; verdict REFUSED (anti-rigging)",
        ),
        Err(e) => GateResult::fail(
            "G3",
            "SOURCE_TREE_UNREADABLE",
            &e,
            if self_mode {
                "self-test source tree unreadable"
            } else {
                "pass --source-tree for foreign subjects (typed, never a silent skip)"
            },
        ),
    }
}

fn gate_g5_self() -> GateResult {
    // Storm: partial-accept (truncated), oversized, bad magic, unknown
    // epoch, digest tamper — every frame must elicit a TYPED refusal,
    // never a panic, never a silent accept.
    let mut good = genesis_header(1);
    let mut wire = [0u8; 64];
    encode(&good, &mut wire).expect("storm encode");
    good.header_digest = u32::from_le_bytes([wire[60], wire[61], wire[62], wire[63]]);

    let mut cases: Vec<(String, Vec<u8>, &'static str)> = Vec::new();
    cases.push((
        "truncated_32B".into(),
        wire[0..32].to_vec(),
        "HeaderTooShort",
    ));
    let mut oversized = vec![0u8; 128];
    oversized[0..64].copy_from_slice(&wire);
    oversized[10] ^= 0xFF; // tamper digest-covered span
    cases.push((
        "oversized_128B_tampered".into(),
        oversized,
        "DigestMismatch",
    ));
    let mut bad_magic = wire;
    bad_magic[0] = 0xDE;
    cases.push(("bad_magic".into(), bad_magic.to_vec(), "BadMagic"));
    let mut bad_epoch = wire;
    bad_epoch[6] = 0x7F; // 0x7FFF is outside the closed Epoch enum (0xFFFF = Retired sentinel, valid)
    bad_epoch[7] = 0xFF;
    cases.push((
        "unknown_epoch".into(),
        bad_epoch.to_vec(),
        "UnknownEpochWire",
    ));
    let mut digest = wire;
    digest[30] = 0x5A;
    cases.push(("digest_tamper".into(), digest.to_vec(), "DigestMismatch"));
    cases.push(("control_valid".into(), wire.to_vec(), "Ok"));

    let mut observed: Vec<String> = Vec::new();
    for (name, payload, expected) in &cases {
        let got_class = match &decode(payload) {
            Ok(_) => "Ok".to_string(),
            Err(e) => match e {
                AoqError::HeaderTooShort { .. } => "HeaderTooShort",
                AoqError::BufferTooShort { .. } => "BufferTooShort",
                AoqError::BadMagic { .. } => "BadMagic",
                AoqError::UnsupportedVersion { .. } => "UnsupportedVersion",
                AoqError::UnknownEpochWire { .. } => "UnknownEpochWire",
                AoqError::DigestMismatch { .. } => "DigestMismatch",
                AoqError::ReservedNotZero => "ReservedNotZero",
                AoqError::FrameOrdinalRegression { .. } => "FrameOrdinalRegression",
                AoqError::SeqGap { .. } => "SeqGap",
                AoqError::KemKernelNotAdmitted { .. } => "KemKernelNotAdmitted",
                AoqError::AeadKernelNotAdmitted { .. } => "AeadKernelNotAdmitted",
            }
            .to_string(),
        };
        observed.push(format!("{}={}", name, got_class));
        if got_class != *expected {
            return GateResult::fail(
                "G5",
                "WRONG_TYPED_CLASS",
                &observed.join(" "),
                &format!(
                    "storm frame {} expected class {}, got {}",
                    name, expected, got_class
                ),
            );
        }
    }
    GateResult::pass(
        "G5",
        "PASS",
        &observed.join(" "),
        "storm feed fully typed; no panic, no silent accept",
    )
}

fn gate_g6_self() -> GateResult {
    GateResult::skip("G6", "SUBJECT_IS_LIBRARY", "cycles=0",
        "subject is an in-process library; kill-cycle machinery is witnessed live in the --court fixture courts (foreign subprocess mode)")
}

// ---------------------------------------------------------------------------
// Foreign subject: subprocess frame protocol (docs/cbmqs-runner.md).
//   PING -> PONG
//   CENSUS -> "CENSUS allocs=<u64> bytes=<u64>" | "ERR NO_CENSUS"
//   OP <n> -> "OPDONE <n>"
//   FRAME <hex> -> "OK" | "ERR <CLASS>"   (never crash)
// ---------------------------------------------------------------------------

struct Subject {
    cmd: String,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<std::io::BufReader<ChildStdout>>,
}

impl Subject {
    fn spawn(cmd: &str) -> Result<Self, String> {
        let mut s = Self {
            cmd: cmd.into(),
            child: None,
            stdin: None,
            stdout: None,
        };
        s.respawn()?;
        Ok(s)
    }

    fn respawn(&mut self) -> Result<(), String> {
        self.child = None;
        self.stdin = None;
        self.stdout = None;
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn failed: {}", e))?;
        let stdin = child.stdin.take().ok_or("no stdin")?;
        let stdout = child.stdout.take().ok_or("no stdout")?;
        self.child = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(std::io::BufReader::new(stdout));
        Ok(())
    }

    /// Send one line, read one reply line. Typed failure on EOF (crash).
    fn exchange(&mut self, line: &str) -> Result<String, String> {
        {
            let stdin = self.stdin.as_mut().ok_or("subject not running")?;
            stdin
                .write_all(line.as_bytes())
                .map_err(|e| format!("write: {}", e))?;
            stdin
                .write_all(b"\n")
                .map_err(|e| format!("write: {}", e))?;
            stdin.flush().map_err(|e| format!("flush: {}", e))?;
        }
        let mut reply = String::new();
        let stdout = self.stdout.as_mut().ok_or("subject not running")?;
        let n = stdout
            .read_line(&mut reply)
            .map_err(|e| format!("read: {}", e))?;
        if n == 0 {
            return Err("EOF: subject crashed or closed stdout".into());
        }
        Ok(reply.trim_end().to_string())
    }

    fn kill(&mut self) {
        if let Some(c) = self.child.as_mut() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for Subject {
    fn drop(&mut self) {
        self.kill();
    }
}

fn parse_census(reply: &str) -> Result<(u64, u64), String> {
    if reply.starts_with("ERR NO_CENSUS") {
        return Err("CONTRACT_UNSUPPORTED".into());
    }
    // CENSUS allocs=<u64> bytes=<u64>
    let allocs = reply
        .split_whitespace()
        .find_map(|t| t.strip_prefix("allocs="))
        .ok_or_else(|| format!("malformed census reply: {:?}", reply))?;
    let bytes = reply
        .split_whitespace()
        .find_map(|t| t.strip_prefix("bytes="))
        .ok_or_else(|| format!("malformed census reply: {:?}", reply))?;
    Ok((
        allocs
            .parse()
            .map_err(|_| format!("bad allocs: {:?}", reply))?,
        bytes
            .parse()
            .map_err(|_| format!("bad bytes: {:?}", reply))?,
    ))
}

fn census_delta(s: &mut Subject, ops: usize) -> Result<(u64, u64), String> {
    let r0 = s.exchange("CENSUS")?;
    let c0 = parse_census(&r0)?;
    let reply = s.exchange(&format!("OP {}", ops))?;
    if !reply.starts_with("OPDONE") {
        return Err(format!("OP reply malformed: {:?}", reply));
    }
    let r1 = s.exchange("CENSUS")?;
    let c1 = parse_census(&r1)?;
    Ok((c1.0.saturating_sub(c0.0), c1.1.saturating_sub(c0.1)))
}

fn gate_g1_foreign(s: &mut Subject, alloc_bound: u64) -> GateResult {
    const OPS: usize = 512;
    match census_delta(s, OPS) {
        Ok((allocs, bytes)) => {
            let per_op = allocs / OPS as u64;
            if allocs <= alloc_bound * OPS as u64 {
                GateResult::pass("G1", "PASS",
                    &format!("ops={} allocs={} bytes={} per_op={} bound_per_op={}", OPS, allocs, bytes, per_op, alloc_bound),
                    "subject-attested census within bound (see docs/cbmqs-runner.md for the attestation disclosure)")
            } else {
                GateResult::fail(
                    "G1",
                    "OVERALLOC",
                    &format!(
                        "ops={} allocs={} bytes={} per_op={} bound_per_op={}",
                        OPS, allocs, bytes, per_op, alloc_bound
                    ),
                    "subject census exceeds the alloc-per-op bound",
                )
            }
        }
        Err(e) if e == "CONTRACT_UNSUPPORTED" => GateResult::fail(
            "G1",
            "CONTRACT_UNSUPPORTED",
            "census frame refused",
            "subject does not implement the CENSUS frame; loudly missing, never a silent skip",
        ),
        Err(e) => GateResult::fail("G1", "PROTOCOL_FAULT", &e, "census exchange failed"),
    }
}

fn gate_g2_foreign(s: &mut Subject, budget_ns: u64) -> GateResult {
    const BATCHES: usize = 16;
    const OPS_PER_BATCH: usize = 64;
    let mut per_op_ns: Vec<u64> = Vec::new();
    for _ in 0..BATCHES {
        let t0 = Instant::now();
        let r = s.exchange(&format!("OP {}", OPS_PER_BATCH));
        match r {
            Ok(reply) if reply.starts_with("OPDONE") => {}
            Ok(reply) => {
                return GateResult::fail("G2", "PROTOCOL_FAULT", &reply, "OP reply malformed")
            }
            Err(e) => return GateResult::fail("G2", "CRASH", &e, "subject died during OP batch"),
        }
        let elapsed = t0.elapsed().as_nanos() as u64 / OPS_PER_BATCH as u64;
        per_op_ns.push(elapsed);
    }
    per_op_ns.sort_unstable();
    let p50 = per_op_ns[per_op_ns.len() / 2];
    let p99 = percentile_p99_u64(&per_op_ns);
    if p99 <= budget_ns {
        GateResult::pass(
            "G2",
            "PASS",
            &format!(
                "batches={} ops_per_batch={} p50_ns_per_op={} p99_ns_per_op={} budget_ns={}",
                BATCHES, OPS_PER_BATCH, p50, p99, budget_ns
            ),
            "timing census within the disclosed budget",
        )
    } else {
        GateResult::fail(
            "G2",
            "TIME_BUDGET_EXCEEDED",
            &format!(
                "batches={} p50_ns_per_op={} p99_ns_per_op={} budget_ns={}",
                BATCHES, p50, p99, budget_ns
            ),
            "p99 per-op wall time exceeded the disclosed budget",
        )
    }
}

enum Expect {
    Ok,
    TypedErr,
    OkOrTypedErr,
}

fn percentile_p99_u64(sorted: &[u64]) -> u64 {
    let idx = ((sorted.len() as u64 * 99) / 100).max(1) as usize - 1;
    sorted[idx.min(sorted.len() - 1)]
}

fn gate_g4_foreign(s: &mut Subject) -> GateResult {
    const OPS: usize = 512;
    let a = census_delta(s, OPS);
    let b = census_delta(s, OPS);
    match (a, b) {
        (Ok(a), Ok(b)) if a == b => GateResult::pass(
            "G4",
            "PASS",
            &format!("run1=({}a,{}B) run2=({}a,{}B)", a.0, a.1, b.0, b.1),
            "two census windows identical across replay",
        ),
        (Ok(a), Ok(b)) => GateResult::fail(
            "G4",
            "CENSUS_REPLAY_DIVERGED",
            &format!("run1=({}a,{}B) run2=({}a,{}B)", a.0, a.1, b.0, b.1),
            "census windows diverged across replay",
        ),
        (Err(e), _) | (_, Err(e)) => {
            GateResult::fail("G4", "PROTOCOL_FAULT", &e, "census replay exchange failed")
        }
    }
}

fn gate_g5_foreign(s: &mut Subject) -> GateResult {
    // Storm payloads: truncated (partial-accept), oversized, bad magic,
    // digest tamper, plus one valid control. Subject codec law (fixture
    // contract): 64-byte frames, "AOQ1" magic; reply OK or ERR <CLASS>.
    let mut valid = vec![0u8; 64];
    valid[0..4].copy_from_slice(b"AOQ1");
    let hex = |b: &[u8]| -> String { b.iter().map(|x| format!("{:02x}", x)).collect() };
    let mut bad_magic = valid.clone();
    bad_magic[0] = 0xDE;
    let mut truncated = valid.clone();
    truncated.truncate(32);
    let mut oversized = vec![0u8; 128];
    oversized[0..64].copy_from_slice(&valid);
    let mut digest_tamper = valid.clone();
    digest_tamper[30] = 0x5A;

    // Frames the contract mandates a typed refusal for (wrong length,
    // wrong magic), plus one law-dependent frame (digest law is
    // subject-specific) and the valid control.
    let frames: Vec<(&str, &Vec<u8>, Expect)> = vec![
        ("control_valid", &valid, Expect::Ok),
        ("truncated_32B", &truncated, Expect::TypedErr),
        ("oversized_128B", &oversized, Expect::TypedErr),
        ("bad_magic", &bad_magic, Expect::TypedErr),
        ("digest_tamper", &digest_tamper, Expect::OkOrTypedErr),
    ];
    let mut observed: Vec<String> = Vec::new();
    let mut fail_msg: Option<String> = None;
    for (name, payload, expect) in &frames {
        let reply = match s.exchange(&format!("FRAME {}", hex(payload))) {
            Ok(r) => r,
            Err(e) => {
                return GateResult::fail(
                    "G5",
                    "CRASH",
                    &format!("{} {}=EOF:{}", observed.join(" "), name, e),
                    "subject crashed or went silent on a storm frame",
                )
            }
        };
        let is_ok = reply == "OK";
        let typed_err = reply.starts_with("ERR ") && reply != "ERR CRASH";
        observed.push(format!("{}={}", name, reply));
        let ok = match expect {
            Expect::Ok => is_ok,
            Expect::TypedErr => typed_err,
            Expect::OkOrTypedErr => is_ok || typed_err,
        };
        if !ok {
            fail_msg = Some(format!(
                "storm-frame law: {} replied {:?}; expected {}",
                name,
                reply,
                match expect {
                    Expect::Ok => "OK",
                    Expect::TypedErr => "a typed ERR <CLASS>",
                    Expect::OkOrTypedErr => "OK or a typed ERR <CLASS>",
                }
            ));
            break;
        }
    }
    if let Some(m) = fail_msg {
        return GateResult::fail(
            "G5",
            "WRONG_TYPED_CLASS",
            &observed.join(" "),
            &format!("{}; contract law violated", m),
        );
    }
    GateResult::pass(
        "G5",
        "PASS",
        &observed.join(" "),
        "storm feed typed per contract; no crash, no silent accept",
    )
}

fn gate_g6_foreign(s: &mut Subject, cycles: usize) -> GateResult {
    let mut witnessed = Vec::new();
    for i in 0..cycles {
        // Prove liveness before the kill, kill -9, then require liveness
        // again after respawn.
        match s.exchange("PING") {
            Ok(r) if r == "PONG" => {}
            Ok(r) => {
                return GateResult::fail(
                    "G6",
                    "PROTOCOL_FAULT",
                    &r,
                    &format!("cycle {} pre-kill ping", i),
                )
            }
            Err(e) => {
                return GateResult::fail("G6", "CRASH", &e, &format!("cycle {} pre-kill ping", i))
            }
        }
        s.kill();
        if let Err(e) = s.respawn() {
            return GateResult::fail("G6", "RESPAWN_FAILED", &e, &format!("cycle {}", i));
        }
        match s.exchange("PING") {
            Ok(r) if r == "PONG" => witnessed.push(format!("cycle{}=ok", i)),
            Ok(r) => {
                return GateResult::fail(
                    "G6",
                    "NO_LIVENESS",
                    &r,
                    &format!("cycle {} post-kill ping", i),
                )
            }
            Err(e) => {
                return GateResult::fail("G6", "CRASH", &e, &format!("cycle {} post-kill ping", i))
            }
        }
    }
    GateResult::pass(
        "G6",
        "PASS",
        &format!("cycles={} {}", cycles, witnessed.join(" ")),
        "kill-cycle: SIGKILL + respawn + frame liveness witnessed every cycle",
    )
}

// ---------------------------------------------------------------------------
// Report assembly + verdict.
// ---------------------------------------------------------------------------

fn select_gates(requested: &str) -> Result<Vec<&'static str>, String> {
    const ALL: &[&str] = &["G1", "G2", "G3", "G4", "G5", "G6", "G7"];
    let mut out = Vec::new();
    for tok in requested.split(',') {
        let t = tok.trim().to_uppercase();
        if t.is_empty() {
            continue;
        }
        if !ALL.contains(&t.as_str()) {
            return Err(format!("unknown gate \"{}\"; valid: G1..G7", tok));
        }
        out.push(ALL.iter().find(|g| **g == t).copied().unwrap());
    }
    Ok(if out.is_empty() { ALL.to_vec() } else { out })
}

fn verdict_from(gates: &[GateResult], refusal: &Option<String>) -> &'static str {
    if refusal.is_some() {
        "REFUSED"
    } else if gates.iter().any(|g| g.status == "FAIL") {
        "FAIL"
    } else {
        "PASS"
    }
}

fn emit_report(
    subject_mode: &'static str,
    subject_id: &str,
    gates: Vec<GateResult>,
    alloc_bound: u64,
    budget_ns: u64,
    requested: &str,
    refusal: Option<String>,
    json_path: Option<&str>,
) -> i32 {
    let verdict = verdict_from(&gates, &refusal);
    let report = Report {
        subject_mode,
        subject_id: subject_id.into(),
        gates,
        alloc_bound_per_op: alloc_bound,
        time_budget_ns: budget_ns,
        gates_requested: requested.into(),
        verdict,
        refusal,
        blake3_hex: String::new(),
    }
    .seal();
    let rendered = report.render();
    // G7 self-check: the runner must be able to verify its own seal.
    let g7 = match Report::verify_hash_law(&rendered) {
        Ok(()) => GateResult::pass(
            "G7",
            "PASS",
            &format!(
                "hash={} algo=blake3-32 runner_version={}",
                &report.blake3_hex[..16],
                RUNNER_VERSION
            ),
            "report seal verifies against the canonical body",
        ),
        Err(e) => GateResult::fail(
            "G7",
            "SEAL_VERIFY_FAILED",
            &e,
            "report seal did not reproduce",
        ),
    };
    let mut report = report;
    if let Some(pos) = report.gates.iter().position(|g| g.gate == "G7") {
        report.gates[pos] = g7;
    } else {
        report.gates.push(g7);
    }
    // Re-seal with the G7 row inside, then re-render.
    let report = report.seal();
    let rendered = report.render();
    if let Some(p) = json_path {
        std::fs::write(p, &rendered)
            .map_err(|e| format!("write {}: {}", p, e))
            .expect("write report");
    }
    println!("{}", rendered);
    match report.verdict {
        "PASS" => 0,
        "FAIL" => 2,
        _ => 3,
    }
}

// ---------------------------------------------------------------------------
// /tmp fixture subjects (real subprocesses; embedded, written at court time).
// ---------------------------------------------------------------------------

const FIXTURE_TEMPLATE: &str = r#"#!/usr/bin/env python3
import sys, tracemalloc
MAGIC = b"AOQ1"
def frame_ok(b):
    return len(b) == 64 and b[0:4] == MAGIC
def main(census=CENSUS_FLAG, op_allocs=ALLOC_FLAG):
    if census:
        tracemalloc.start()
    base_allocs = 0
    if census:
        s0 = tracemalloc.take_snapshot()
        base_allocs = sum(st.count for st in s0.statistics("lineno"))
    for line in sys.stdin:
        line = line.strip()
        if line == "PING":
            print("PONG", flush=True)
        elif line == "CENSUS":
            if not census:
                print("ERR NO_CENSUS", flush=True)
            else:
                s = tracemalloc.take_snapshot()
                total = sum(st.count for st in s.statistics("lineno"))
                print("CENSUS allocs=%d bytes=0" % max(0, total - base_allocs), flush=True)
        elif line.startswith("OP "):
            n = int(line.split()[1])
            for _ in range(n):
                if op_allocs:
                    blob = bytes(4096)  # deliberate per-op heap traffic
                    _ = len(blob)
                else:
                    _ = (4096 >> 3) & 0xFF  # small-int arithmetic; no heap per op
            print("OPDONE %d" % n, flush=True)
        elif line.startswith("FRAME "):
            try:
                b = bytes.fromhex(line[6:])
            except ValueError:
                print("ERR BAD_HEX", flush=True); continue
            if frame_ok(b):
                print("OK", flush=True)
            elif len(b) != 64:
                print("ERR BAD_LENGTH", flush=True)
            else:
                print("ERR BAD_MAGIC", flush=True)
        else:
            print("ERR UNKNOWN_CMD", flush=True)

main()
"#;

fn write_fixture(path: &str, census: bool, op_allocs: bool) -> Result<(), String> {
    let body = FIXTURE_TEMPLATE
        .replace("CENSUS_FLAG", if census { "True" } else { "False" })
        .replace("ALLOC_FLAG", if op_allocs { "True" } else { "False" });
    std::fs::write(path, body).map_err(|e| format!("write {}: {}", path, e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("chmod {}: {}", path, e))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Modes.
// ---------------------------------------------------------------------------

fn run_self_test(alloc_bound: u64, budget_ns: u64, requested: &str, json: Option<&str>) -> i32 {
    let _ = alloc_bound; // self-test bounds are the codec's own laws (G1 bound 0)
    let gates = select_gates(requested).expect("gate selection");
    let mut out = Vec::new();
    for g in gates {
        out.push(match g {
            "G1" => gate_g1_self(),
            "G2" => gate_g2_self(),
            "G3" => gate_g3_scan(&format!("{}/src", env!("CARGO_MANIFEST_DIR")), true),
            "G4" => gate_g4_self(),
            "G5" => gate_g5_self(),
            "G6" => gate_g6_self(),
            "G7" => GateResult::skip(
                "G7",
                "SEALED_AT_EMIT",
                "",
                "seal computed when the report is emitted",
            ),
            _ => unreachable!(),
        });
    }
    emit_report(
        "self-test",
        &format!("cgap-aoq@{}", env!("CARGO_PKG_VERSION")),
        out,
        0,
        budget_ns,
        requested,
        None,
        json,
    )
}

fn run_foreign(
    cmd: &str,
    source_tree: Option<&str>,
    alloc_bound: u64,
    budget_ns: u64,
    requested: &str,
    json: Option<&str>,
) -> i32 {
    let gates = match select_gates(requested) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("cbmqs-runner: {}", e);
            return 1;
        }
    };
    // G3 first: a MOCK_DETECTED refusal short-circuits the run.
    let (g3_result, refusal) = match source_tree {
        Some(tree) => {
            let r = gate_g3_scan(tree, false);
            let refused = r.kind == "MOCK_DETECTED";
            let measured = r.measured.clone();
            (
                Some(r),
                if refused {
                    Some(format!("MOCK_DETECTED: {}", measured))
                } else {
                    None
                },
            )
        }
        None => (
            Some(GateResult::fail(
                "G3",
                "SOURCE_TREE_REQUIRED",
                "no --source-tree given",
                "mock scan requires the subject source tree; loudly missing",
            )),
            None,
        ),
    };
    if refusal.is_some() {
        return emit_report(
            "subprocess",
            cmd,
            vec![g3_result.unwrap()],
            alloc_bound,
            budget_ns,
            requested,
            refusal,
            json,
        );
    }

    let mut s = match Subject::spawn(cmd) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cbmqs-runner: subject spawn failed: {}", e);
            return 1;
        }
    };
    let mut out: Vec<GateResult> = Vec::new();
    for g in gates {
        out.push(match g {
            "G1" => gate_g1_foreign(&mut s, alloc_bound),
            "G2" => gate_g2_foreign(&mut s, budget_ns),
            "G3" => g3_result.clone().unwrap(),
            "G4" => gate_g4_foreign(&mut s),
            "G5" => gate_g5_foreign(&mut s),
            "G6" => gate_g6_foreign(&mut s, 3),
            "G7" => GateResult::skip(
                "G7",
                "SEALED_AT_EMIT",
                "",
                "seal computed when the report is emitted",
            ),
            _ => unreachable!(),
        });
    }
    emit_report(
        "subprocess",
        cmd,
        out,
        alloc_bound,
        budget_ns,
        requested,
        None,
        json,
    )
}

/// Non-vacuity court: every gate must flip somewhere. Exit 0 = court pass.
fn run_court() -> i32 {
    let dir = "/tmp/cbmqs_fixtures";
    let _ = std::fs::create_dir_all(dir);
    let good = format!("{}/cbmqs_subject_good.py", dir);
    let over = format!("{}/cbmqs_subject_overalloc.py", dir);
    let nocen = format!("{}/cbmqs_subject_nocensus.py", dir);
    if let Err(e) = write_fixture(&good, true, false) {
        eprintln!("court: {}", e);
        return 1;
    }
    if let Err(e) = write_fixture(&over, true, true) {
        eprintln!("court: {}", e);
        return 1;
    }
    if let Err(e) = write_fixture(&nocen, false, false) {
        eprintln!("court: {}", e);
        return 1;
    }

    let mut failures: Vec<String> = Vec::new();

    // C1: self-test passes its own standard (typed G6 SKIP disclosed).
    let rc = run_self_test(
        0,
        10_000,
        "G1,G2,G3,G4,G5,G6,G7",
        Some("/tmp/cbmqs_self_report.json"),
    );
    if rc != 0 {
        failures.push(format!("C1 self-test exit={} (expected 0)", rc));
    }

    // C2: overalloc fixture fails G1 typed OVERALLOC with real numbers.
    let rc = run_foreign(
        &over,
        Some(dir),
        0,
        100_000_000,
        "G1",
        Some("/tmp/cbmqs_overalloc_report.json"),
    );
    let over_json = std::fs::read_to_string("/tmp/cbmqs_overalloc_report.json").unwrap_or_default();
    if rc != 2 || !over_json.contains("\"kind\": \"OVERALLOC\"") {
        failures.push(format!(
            "C2 overalloc exit={} json_has_OVERALLOC={}",
            rc,
            over_json.contains("\"kind\": \"OVERALLOC\"")
        ));
    }

    // C3: missing-census fixture fails G1 typed CONTRACT_UNSUPPORTED.
    let rc = run_foreign(
        &nocen,
        Some(dir),
        0,
        100_000_000,
        "G1",
        Some("/tmp/cbmqs_nocensus_report.json"),
    );
    let nocen_json = std::fs::read_to_string("/tmp/cbmqs_nocensus_report.json").unwrap_or_default();
    if rc != 2 || !nocen_json.contains("CONTRACT_UNSUPPORTED") {
        failures.push(format!(
            "C3 nocensus exit={} json_has_CONTRACT_UNSUPPORTED={}",
            rc,
            nocen_json.contains("CONTRACT_UNSUPPORTED")
        ));
    }

    // C4: bound-flip pair — the same overalloc subject FAILS at bound 0
    // and PASSES at bound 2 (the bound actually binds; a gate that cannot
    // flip is decoration).
    let rc0 = run_foreign(
        &over,
        Some(dir),
        0,
        100_000_000,
        "G1",
        Some("/tmp/cbmqs_good_b0.json"),
    );
    let rc2 = run_foreign(
        &over,
        Some(dir),
        2,
        100_000_000,
        "G1",
        Some("/tmp/cbmqs_good_b8.json"),
    );
    if rc0 != 2 || rc2 != 0 {
        failures.push(format!(
            "C4 bound-flip pair did not flip: bound0_exit={} bound2_exit={}",
            rc0, rc2
        ));
    }

    // C5: good fixture — storm feed + kill-cycles witness live machinery.
    let rc = run_foreign(
        &good,
        Some(dir),
        8,
        100_000_000,
        "G5,G6",
        Some("/tmp/cbmqs_good_storm.json"),
    );
    let storm_json = std::fs::read_to_string("/tmp/cbmqs_good_storm.json").unwrap_or_default();
    if rc != 0
        || !storm_json.contains("\"gate\": \"G6\"")
        || storm_json.contains("\"kind\": \"CRASH\"")
    {
        failures.push(format!(
            "C5 storm/kill-cycles exit={} G6_present={}",
            rc,
            storm_json.contains("\"gate\": \"G6\"")
        ));
    }

    // C6: mock tree refusal.
    let mock_dir = "/tmp/cbmqs_mocktree";
    let _ = std::fs::create_dir_all(mock_dir);
    let _ = std::fs::write(
        format!("{}/subject.rs", mock_dir),
        "use mockall::predicate::*;\n",
    );
    let rc = run_foreign(
        &good,
        Some(mock_dir),
        8,
        100_000_000,
        "G3",
        Some("/tmp/cbmqs_mock_report.json"),
    );
    let mock_json = std::fs::read_to_string("/tmp/cbmqs_mock_report.json").unwrap_or_default();
    if rc != 3 || !mock_json.contains("MOCK_DETECTED") {
        failures.push(format!(
            "C6 mock refusal exit={} json_has_MOCK_DETECTED={}",
            rc,
            mock_json.contains("MOCK_DETECTED")
        ));
    }

    // C7: tamper court — a one-bit flip must break the seal.
    let sealed = std::fs::read_to_string("/tmp/cbmqs_self_report.json").unwrap_or_default();
    let tampered = sealed.replacen("cgap-aoq@", "cgap-aoq!@", 1);
    let verdict = match Report::verify_hash_law(&tampered) {
        Err(_) => "REFUSED",
        Ok(()) => "ACCEPTED",
    };
    if verdict != "REFUSED" {
        failures.push("C7 tampered report verified as clean; seal is vacuous".into());
    }

    if failures.is_empty() {
        println!("CBMQS COURT PASS: self-test green; G1 OVERALLOC/CONTRACT_UNSUPPORTED typed fails witnessed; bound-flip pair flipped; storm+kill-cycles live; MOCK_DETECTED refusal fired; tamper court refused.");
        0
    } else {
        for f in &failures {
            eprintln!("COURT FAIL: {}", f);
        }
        1
    }
}

fn usage() {
    eprintln!(
        "cbmqs-runner {} — CBMQS-7G/1 bake-off runner",
        RUNNER_VERSION
    );
    eprintln!("  (no args)                     self-test against cgap-aoq");
    eprintln!("  --court                       non-vacuity court over /tmp fixtures");
    eprintln!("  --subject-cmd <shell cmd>     foreign subprocess subject");
    eprintln!("  --source-tree <dir>           tree scanned by G3 (required for foreign)");
    eprintln!("  --gates G1,G5                 gate subset (default all 7)");
    eprintln!("  --alloc-bound <u64>           foreign G1 allocs-per-op bound (default 0)");
    eprintln!("  --time-budget-ns <u64>        G2 per-op budget (default 10000000)");
    eprintln!("  --json <path>                 also write the report JSON here");
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut subject_cmd: Option<String> = None;
    let mut source_tree: Option<String> = None;
    let mut gates = "G1,G2,G3,G4,G5,G6,G7".to_string();
    let mut alloc_bound: u64 = 0;
    let mut budget_ns: u64 = 10_000_000;
    let mut json: Option<String> = None;
    let mut court = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--court" => court = true,
            "--subject-cmd" => subject_cmd = args.next(),
            "--source-tree" => source_tree = args.next(),
            "--gates" => gates = args.next().unwrap_or_else(|| gates.clone()),
            "--alloc-bound" => {
                alloc_bound = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(alloc_bound);
            }
            "--time-budget-ns" => {
                budget_ns = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(budget_ns);
            }
            "--json" => json = args.next(),
            "--help" | "-h" => {
                usage();
                return;
            }
            other => {
                eprintln!("cbmqs-runner: unknown arg {}", other);
                usage();
                std::process::exit(1);
            }
        }
    }
    let code = if court {
        run_court()
    } else if let Some(cmd) = subject_cmd {
        run_foreign(
            &cmd,
            source_tree.as_deref(),
            alloc_bound,
            budget_ns,
            &gates,
            json.as_deref(),
        )
    } else {
        run_self_test(alloc_bound, budget_ns, &gates, json.as_deref())
    };
    std::process::exit(code);
}
