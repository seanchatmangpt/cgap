//! Certificate file I/O + schema-key refusal law (P5 phase 2; DOD4 repair).
//!
//! Emit writes the canonical certificate JSON to `<path>` and the hex
//! BLAKE3-256 seal to the sidecar `<path>.seal`. Verify reads both back
//! and refuses typed (fail-closed) on: unknown schema_version /
//! canonical_form / seal_algorithm; malformed canonical form (duplicate
//! keys, reordering, prepended bytes, whitespace outside strings,
//! non-canonical escapes); body-hash mismatch (the PRESENTED body's
//! gates, rehashed under the S6 law, must equal the presented
//! input_hash); seal mismatch; missing sidecar; and the RunSeqCollision
//! identity law (same run_seq binds one input_hash -- the S6-derived
//! table's).
//!
//! DOD4 repair laws:
//! * H1 -- the old string-scrape extractors took the FIRST occurrence of
//!   a key, so attacker-prepended bytes hijacked parsed identity (D4
//!   attack A12 verified rc=0 before the repair). Replaced by a strict
//!   single-pass canonical parser with FIXED key order: each key
//!   appears at most once by construction; any other byte sequence is
//!   Malformed.
//! * H2 -- the unkeyed seal is forgeable by resealing (D4 attack A13
//!   verified rc=0 before the repair). Verify now RECOMPUTES the
//!   content hash over the presented body's gates (same S6 law as
//!   emit) and refuses unless it equals the presented input_hash,
//!   BEFORE the seal check. Resealing forgery dies here: the patched
//!   body hashes differently.
//! * H3 -- the old envelope spliced CLI schema keys in front of a
//!   kernel body that already carried them, producing duplicate keys
//!   with conflicting values. Emit now REUSES the kernel body's leading
//!   `schema_version` and REPLACES the kernel's trailing
//!   canonical_form/seal_algorithm values with the CLI constants --
//!   exactly one copy of each key.

use std::fmt;

use cgap_aoq::rfc0089_certificate::{GateVerdict, Verdict, VerdictTable};

/// Pinned envelope keys (fail-closed: unknown values are refused).
pub const SCHEMA_VERSION: u64 = 1;
pub const CANONICAL_FORM: &str = "cbmqs-certificate-v1";
pub const SEAL_ALGORITHM: &str = "BLAKE3-256-hex";

/// The kernel's trailing schema-key tail, verbatim as
/// `cgap_aoq::rfc0089_certificate::certificate_json` emits it.
const KERNEL_SCHEMA_TAIL: &str =
    "\"canonical_form\":\"fixed-declaration-order-v1\",\"seal_algorithm\":\"BLAKE3-256\"";

/// Typed refusal surface. Every variant maps to exit 2 except
/// `CertificateIncomplete` (exit 3, loud skip, no artifact).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// schema_version this binary does not know.
    SchemaVersionUnknown { observed: String },
    /// canonical_form this binary does not know.
    CanonicalFormUnknown { observed: String },
    /// seal_algorithm this binary does not know.
    SealAlgorithmUnknown { observed: String },
    /// Canonical bytes are not parseable as the pinned canonical form
    /// (duplicate key, reorder, prepend, whitespace, bad escape, ...).
    Malformed { detail: String },
    /// Sidecar `<path>.seal` is absent or unreadable.
    MissingSeal { path: String },
    /// Recomputed BLAKE3-256 over canonical bytes != presented seal.
    SealMismatch { expected: String, observed: String },
    /// (run_seq, input_hash) identity law: the presented certificate
    /// binds run_seq to an input_hash the S6-derived table for that
    /// run_seq does not carry.
    RunSeqCollision {
        run_seq: u64,
        presented_input_hash: String,
        derived_input_hash: String,
    },
    /// DOD4 H2 body-binding law: rehashing the PRESENTED body's gates
    /// under the S6 law does not reproduce the presented input_hash.
    /// Closes the reseal forgery (patch a gate, recompute the seal).
    BodyHashMismatch {
        presented_input_hash: String,
        body_input_hash: String,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::SchemaVersionUnknown { observed } => write!(
                f,
                "REFUSED schema_version unknown: {observed} (known: {SCHEMA_VERSION})"
            ),
            Refusal::CanonicalFormUnknown { observed } => write!(
                f,
                "REFUSED canonical_form unknown: {observed} (known: {CANONICAL_FORM})"
            ),
            Refusal::SealAlgorithmUnknown { observed } => write!(
                f,
                "REFUSED seal_algorithm unknown: {observed} (known: {SEAL_ALGORITHM})"
            ),
            Refusal::Malformed { detail } => write!(f, "REFUSED malformed: {detail}"),
            Refusal::MissingSeal { path } => write!(f, "REFUSED missing seal sidecar: {path}"),
            Refusal::SealMismatch { expected, observed } => write!(
                f,
                "REFUSED seal mismatch: expected {expected}, observed {observed}"
            ),
            Refusal::RunSeqCollision {
                run_seq,
                presented_input_hash,
                derived_input_hash,
            } => write!(
                f,
                "REFUSED RunSeqCollision: run_seq {run_seq} presented input_hash \
                 {presented_input_hash} but the §6-derived table carries {derived_input_hash}"
            ),
            Refusal::BodyHashMismatch {
                presented_input_hash,
                body_input_hash,
            } => write!(
                f,
                "REFUSED BodyHashMismatch: rehashing the presented body's gates under \
                 the §6 law yields {body_input_hash}, not the presented input_hash \
                 {presented_input_hash} (reseal-forgery refusal)"
            ),
        }
    }
}

// ------------------------------------------------ envelope construction

/// Canonical certificate envelope (DOD4 H3): the kernel body's leading
/// `schema_version` is REUSED (value must equal the CLI constant) and
/// the kernel's trailing canonical_form/seal_algorithm values are
/// REPLACED with the CLI constants. Exactly one copy of each key. Fixed
/// key order, no whitespace -- the file bytes ARE the sealed canonical
/// string.
pub fn envelope_json(cert_json: &str) -> Result<String, Refusal> {
    if !(cert_json.starts_with('{') && cert_json.ends_with('}')) {
        return Err(Refusal::Malformed {
            detail: "kernel certificate_json is not a JSON object".into(),
        });
    }
    // The kernel body is a complete JSON object: strip BOTH braces so
    // `inner` is the raw key/value payload (the kernel's own closing
    // brace must not survive into the re-wrapped envelope).
    let inner = &cert_json[1..cert_json.len() - 1];
    let schema_prefix = format!("\"schema_version\":{},", SCHEMA_VERSION);
    if !inner.starts_with(&schema_prefix) {
        return Err(Refusal::Malformed {
            detail: "kernel body does not lead with the pinned schema_version".into(),
        });
    }
    let core = inner
        .strip_suffix(&format!(",{KERNEL_SCHEMA_TAIL}"))
        .ok_or_else(|| Refusal::Malformed {
            detail: "kernel body missing the canonical_form/seal_algorithm tail".into(),
        })?;
    Ok(format!(
        "{{{core},\"canonical_form\":\"{CANONICAL_FORM}\",\
          \"seal_algorithm\":\"{SEAL_ALGORITHM}\"}}"
    ))
}

/// Seal = hex(BLAKE3-256 over the canonical envelope bytes).
pub fn seal_over(canonical: &[u8]) -> String {
    cgap_blake3::hash(canonical)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ------------------------------------- strict canonical JSON parser (H1)

/// Verdict kind as parsed from the presented body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedKind {
    Pass,
    Fail,
    Skip,
}

/// One gate row parsed from the presented body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedGate {
    pub id: String,
    pub composes: String,
    pub kind: ParsedKind,
    /// `why` for fail / `reason` for skip; None for pass.
    pub detail: Option<String>,
    pub measured: String,
    pub threshold: String,
}

/// What verify needs from the canonical bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCertificate {
    pub run_seq: u64,
    pub input_hash: String,
    pub gates: Vec<ParsedGate>,
    pub overall: String,
}

/// Byte cursor over the canonical bytes. UTF-8 string payloads pass
/// through byte-wise: multibyte sequences never collide with the ASCII
/// structural bytes (`"`, `:`, `,`, `{`, `[`).
struct Cur<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Cur<'a> {
    fn new(b: &'a [u8]) -> Self {
        Cur { b, i: 0 }
    }
    /// Literal bytes, no whitespace allowed (canonical form).
    fn lit(&mut self, s: &str) -> Result<(), Refusal> {
        let e = self.i + s.len();
        if e <= self.b.len() && &self.b[self.i..e] == s.as_bytes() {
            self.i = e;
            Ok(())
        } else {
            Err(Refusal::Malformed {
                detail: format!("expected {s:?} at byte {}", self.i),
            })
        }
    }
    /// Canonical string: scan bytes, unescape ONLY the escapes
    /// `json_escape` produces (`\" \\ \n \r \t \uXXXX` for < 0x20).
    /// A `\uXXXX` naming a codepoint >= 0x20 is non-canonical ->
    /// Malformed (kills unicode-escape re-serialization attacks).
    fn string(&mut self) -> Result<String, Refusal> {
        self.lit("\"")?;
        let mut out = String::new();
        loop {
            let c = *self.b.get(self.i).ok_or(Refusal::Malformed {
                detail: "unterminated string".into(),
            })?;
            self.i += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or(Refusal::Malformed {
                        detail: "truncated escape".into(),
                    })?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let h = self.hex4()?;
                            let ch = char::from_u32(h).ok_or(Refusal::Malformed {
                                detail: "invalid \\u escape".into(),
                            })?;
                            if !ch.is_control() {
                                return Err(Refusal::Malformed {
                                    detail: "non-canonical \\u escape for a printable \
                                             char (canonical form emits it literally)"
                                        .into(),
                                });
                            }
                            out.push(ch);
                        }
                        _ => {
                            return Err(Refusal::Malformed {
                                detail: "non-canonical escape".into(),
                            })
                        }
                    }
                }
                0x00..=0x1f => {
                    return Err(Refusal::Malformed {
                        detail: "raw control byte in string".into(),
                    })
                }
                _ => {
                    // Multibyte UTF-8: copy the whole sequence.
                    let start = self.i - 1;
                    let len = utf8_len(c);
                    let end = start + len;
                    if len > 1 {
                        if end > self.b.len() {
                            return Err(Refusal::Malformed {
                                detail: "truncated UTF-8 sequence".into(),
                            });
                        }
                        self.i = end;
                    }
                    out.push_str(std::str::from_utf8(&self.b[start..self.i]).map_err(|_| {
                        Refusal::Malformed {
                            detail: "invalid UTF-8 in string".into(),
                        }
                    })?);
                }
            }
        }
    }
    fn hex4(&mut self) -> Result<u32, Refusal> {
        if self.i + 4 > self.b.len() {
            return Err(Refusal::Malformed {
                detail: "truncated \\u escape".into(),
            });
        }
        let s =
            std::str::from_utf8(&self.b[self.i..self.i + 4]).map_err(|_| Refusal::Malformed {
                detail: "invalid \\u hex".into(),
            })?;
        let v = u32::from_str_radix(s, 16).map_err(|_| Refusal::Malformed {
            detail: "invalid \\u hex".into(),
        })?;
        self.i += 4;
        Ok(v)
    }
    fn u64(&mut self) -> Result<u64, Refusal> {
        let start = self.i;
        while matches!(self.b.get(self.i), Some(b'0'..=b'9')) {
            self.i += 1;
        }
        if self.i == start {
            return Err(Refusal::Malformed {
                detail: "expected digits".into(),
            });
        }
        std::str::from_utf8(&self.b[start..self.i])
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or(Refusal::Malformed {
                detail: "unparsable u64".into(),
            })
    }
}

fn utf8_len(first: u8) -> usize {
    match first {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => 1, // invalid lead; from_utf8 rejects downstream
    }
}

/// Parse one gate object: fixed key order id, composes, verdict,
/// measured, threshold; verdict is {"kind":"pass"} / {"kind":"fail",
/// "why":...} / {"kind":"skip","reason":...}.
fn parse_gate(c: &mut Cur) -> Result<ParsedGate, Refusal> {
    c.lit("{")?;
    c.lit("\"id\":")?;
    let id = c.string()?;
    c.lit(",\"composes\":")?;
    let composes = c.string()?;
    c.lit(",\"verdict\":{\"kind\":")?;
    let kind = match c.b.get(c.i) {
        Some(b'"') => {
            let v = c.string()?;
            match v.as_str() {
                "pass" => ParsedKind::Pass,
                "fail" => ParsedKind::Fail,
                "skip" => ParsedKind::Skip,
                other => {
                    return Err(Refusal::Malformed {
                        detail: format!("unknown verdict kind {other:?}"),
                    })
                }
            }
        }
        _ => {
            return Err(Refusal::Malformed {
                detail: "verdict kind must be a string".into(),
            })
        }
    };
    let detail = match kind {
        ParsedKind::Pass => None,
        ParsedKind::Fail => {
            c.lit(",\"why\":")?;
            Some(c.string()?)
        }
        ParsedKind::Skip => {
            c.lit(",\"reason\":")?;
            Some(c.string()?)
        }
    };
    c.lit("},\"measured\":")?;
    let measured = c.string()?;
    c.lit(",\"threshold\":")?;
    let threshold = c.string()?;
    c.lit("}")?;
    Ok(ParsedGate {
        id,
        composes,
        kind,
        detail,
        measured,
        threshold,
    })
}

/// Fail-closed envelope parse (DOD4 H1): strict single-pass canonical
/// parser. FIXED top-level key order = schema_version, spec, run_id,
/// timestamp_source, run_seq, input_hash, gates, overall,
/// evidence_receipts, canonical_form, seal_algorithm. Each key appears
/// at most once BY CONSTRUCTION (duplicate-key / prepend hijack is a
/// byte mismatch -> Malformed). Unknown schema keys are typed refusals;
/// missing/unparsable fields are Malformed. Never guess, never default.
pub fn parse_envelope(canon: &str) -> Result<ParsedCertificate, Refusal> {
    let mut c = Cur::new(canon.as_bytes());
    c.lit("{\"schema_version\":")?;
    let schema = c.u64()?;
    if schema != SCHEMA_VERSION {
        return Err(Refusal::SchemaVersionUnknown {
            observed: schema.to_string(),
        });
    }
    c.lit(",\"spec\":")?;
    let _spec = c.string()?;
    c.lit(",\"run_id\":")?;
    let _run_id = c.string()?;
    c.lit(",\"timestamp_source\":")?;
    let ts = c.string()?;
    if ts != "NONE" {
        return Err(Refusal::Malformed {
            detail: format!("timestamp_source must be NONE, got {ts:?}"),
        });
    }
    c.lit(",\"run_seq\":")?;
    let run_seq = c.u64()?;
    c.lit(",\"input_hash\":")?;
    let input_hash = c.string()?;
    if input_hash.len() != 64 || !input_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Refusal::Malformed {
            detail: "input_hash must be 64 hex chars".into(),
        });
    }
    if input_hash.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err(Refusal::Malformed {
            detail: "input_hash must be lowercase hex".into(),
        });
    }
    c.lit(",\"gates\":[")?;
    let mut gates = Vec::new();
    loop {
        gates.push(parse_gate(&mut c)?);
        match c.b.get(c.i) {
            Some(b',') => c.i += 1,
            Some(b']') => {
                c.i += 1;
                break;
            }
            _ => {
                return Err(Refusal::Malformed {
                    detail: "unterminated gates array".into(),
                })
            }
        }
    }
    if gates.is_empty() {
        return Err(Refusal::Malformed {
            detail: "gates array empty".into(),
        });
    }
    c.lit(",\"overall\":")?;
    let overall = c.string()?;
    if overall != "PASS" && overall != "DISQUALIFIED" {
        return Err(Refusal::Malformed {
            detail: format!("unknown overall {overall:?}"),
        });
    }
    c.lit(",\"evidence_receipts\":[")?;
    if c.b.get(c.i) == Some(&b']') {
        c.i += 1;
    } else {
        loop {
            let _r = c.string()?;
            match c.b.get(c.i) {
                Some(b',') => c.i += 1,
                Some(b']') => {
                    c.i += 1;
                    break;
                }
                _ => {
                    return Err(Refusal::Malformed {
                        detail: "unterminated evidence_receipts".into(),
                    })
                }
            }
        }
    }
    c.lit(",\"canonical_form\":")?;
    let form = c.string()?;
    if form != CANONICAL_FORM {
        return Err(Refusal::CanonicalFormUnknown { observed: form });
    }
    c.lit(",\"seal_algorithm\":")?;
    let alg = c.string()?;
    if alg != SEAL_ALGORITHM {
        return Err(Refusal::SealAlgorithmUnknown { observed: alg });
    }
    c.lit("}")?;
    if c.i != c.b.len() {
        return Err(Refusal::Malformed {
            detail: format!(
                "{} trailing bytes after the canonical object",
                c.b.len() - c.i
            ),
        });
    }
    Ok(ParsedCertificate {
        run_seq,
        input_hash,
        gates,
        overall,
    })
}

// -------------------------------------------------- body-hash law (H2)

/// DOD4 H2: reconstruct the §6 verdict table from the PRESENTED body's
/// gates (the same kernel types `canonical_table_json` serializes), so
/// the body can be rehashed under the identical law used at emit. A
/// patched body (e.g. a gate fail->pass flip) hashes differently ->
/// BodyHashMismatch -> refusal BEFORE the (forgeable, unkeyed) seal is
/// ever consulted.
pub fn rebuild_table(p: &ParsedCertificate) -> Result<VerdictTable, Refusal> {
    let gates = p
        .gates
        .iter()
        .map(|g| GateVerdict {
            id: Box::leak(g.id.clone().into_boxed_str()) as &'static str,
            composes: Box::leak(g.composes.clone().into_boxed_str()) as &'static str,
            verdict: match &g.kind {
                ParsedKind::Pass => Verdict::Pass,
                ParsedKind::Fail => Verdict::Fail {
                    why: g.detail.clone().unwrap_or_default(),
                },
                ParsedKind::Skip => Verdict::Skip {
                    reason: g.detail.clone().unwrap_or_default(),
                },
            },
            measured: g.measured.clone(),
            threshold: Box::leak(g.threshold.clone().into_boxed_str()) as &'static str,
        })
        .collect();
    Ok(VerdictTable {
        run_seq: p.run_seq,
        gates,
    })
}

/// Read + validate the sidecar seal: lowercase hex, exactly 64 chars.
pub fn read_seal(cert_path: &str) -> Result<String, Refusal> {
    let seal_path = format!("{cert_path}.seal");
    let raw = std::fs::read_to_string(&seal_path).map_err(|_| Refusal::MissingSeal {
        path: seal_path.clone(),
    })?;
    let seal = raw.trim();
    if seal.len() != 64 || !seal.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Refusal::SealMismatch {
            expected: "64 lowercase hex chars".into(),
            observed: seal.to_string(),
        });
    }
    Ok(seal.to_ascii_lowercase())
}

/// Lowercase hex codec.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
