//! Register-file ABI + branchless-selection kernel for the SIMD unifier.
//! Spec: `docs/jira/v26.11.1/lanes/A12-simd-unifier.md` Sections 4.2-4.4
//! and 7, with the statement ABI **cited, not redefined** from
//! `docs/jira/v26.11.1/lanes/A11-bn3-arena.md` Sections 4.1-4.3
//! (`crate::sigil::BN3Statement` / term words, frozen in `sigil.rs`).
//!
//! Landed state (v26.11.2 convergence): the register ABI (band partition,
//! widths, typed error surface) is pinned by compile-time asserts; the core
//! engines are in this file — [`RuleCompiler`] (rule words -> register
//! assignments), the per-atom mask kernels over columnar term words, the
//! join probe (RADIXFIX: radix-mask default, nested-loop leg retained and
//! court-guarded in `chains-bn3.rs`), selection emission, and the fixpoint
//! driver (W6-BFX: [`SplitStore`] borrowed-prefix + owned-delta-tail view)
//! that feeds the [`crate::project::ConsequentProjector`]. Posture:
//! W6-BN3NS `no_std` (alloc collections, no std dep); the fixpoint is NOT
//! budget-0 (H8 honesty court, `noalloc.rs` counting allocator); the AE6
//! behavioral patent-gate delta is the campaign G7 admission surface.
//!
//! Authority note (spec A11 Section 5): A11 guarantees the 8-bit term-word
//! payload transports a register number `R_n`, `n` in `0..=255`; the
//! *binding-band* semantics — which registers are constants, variables,
//! and scratch — are adjudicated here (A12 is the register authority).
//!
//! Recorded divergences (spec yields to authoritative lanes / reality):
//!
//! 1. **Input shape**: spec 4.3 step 1 says Arrow record batches
//!    (`UInt8`/`UInt32`/dictionary columns). `cgap-bn3` has no
//!    `cgap-arrow` dependency and the manifest is coordinator-owned, so
//!    Wave 2 evaluates over [`StatementColumns`] — SoA slices of A11 term
//!    words. The Arrow binding is the Wave-3 integration seam; the M12-6
//!    determinism court already pins chunk-partition independence, which
//!    is the property Arrow chunking needs.
//! 2. **Statement layout**: spec 4.1's 24-byte record
//!    (`s: u64, p: u64, object: u32, flags: u16`) is superseded by the
//!    authoritative A11 layout (three 64-bit term words, no pad field).
//!    Consequence: the spec 7.3 M12-4 pad-corruption mutation has no
//!    subject under A11 — recorded N/A, not silently dropped.
//! 3. **Join micro-architecture**: spec 4.3 step 3 scatters bound values
//!    into scratch registers via a masked gather. Wave 2 implements the
//!    spec 3.2 v1 strategy — nested-loop with mask AND — where the join
//!    probe is a value-equality test of the candidate row against the
//!    match's bound values. The masked-gather form is the BM0-gated
//!    Wave-3 micro-architecture; M12-3's join-swap mutation class still
//!    applies (swapping the probe direction flips the differential court).
//! 4. **Compaction**: spec 4.3 step 4 wants prefix-sum branch-free
//!    compaction. Wave 2 compacts by mask test during match construction
//!    (correctness first); the prefix-sum form lands with the BM0 court
//!    in Wave 3, where it is measurable.
//!
//! Wave 4 (hardening + DoD closure):
//!
//! 5. **Zero-allocation scan+project path** (spec 7.4, DoD 6): every heap
//!    buffer the per-chunk evaluation touches now lives in
//!    [`EvalScratch`], reused across chunks, rounds, and court runs; the
//!    projector gained [`crate::project::ConsequentProjector::emit_into`]
//!    over caller-owned [`crate::project::DeltaRows`] + canonical-byte
//!    buffers. After a warmup run sizes the buffers on identical input, a
//!    second scan+project run touches the allocator zero times — courted
//!    below with a counting global allocator (house pattern:
//!    `crates/cgap-parquet/tests/page_zero_alloc.rs`). The mutation "put
//!    per-call allocation back" MUST flip that court. Evidence standing
//!    (Wave-5, 2026-11-01): the Wave-4 lane report this note previously
//!    cited is absent from the repo (`docs/jira/v26.11.1/waves/` holds
//!    only the a11/a17/a20 receipts), so the claim rests on the Wave-5
//!    re-execution: shim `W5-M7` re-introduced a per-call allocation in
//!    `atom_local_match_into` and flipped
//!    `zero_alloc_court_scan_project_hot_path_allocates_zero_after_warmup`
//!    (rc=101), with byte-exact restore. The full Wave-5 shim ledger
//!    (M12-1/3/5/6/7/8 + 7.4 + A20-M10/M11 replays, 8/8 FLIP, 0 vacuous)
//!    is recorded in the lane's Wave-5 report.
//! 6. **no_std decision (DoD 1, explicit — registered, not hand-waved)**:
//!    `cgap-bn3` targets `std` for v1. The BM0 perf court needs hosted
//!    hardware counters, the digest needs `Vec`, and the court binary
//!    needs the std test harness. The seam is real, not aspirational:
//!    after Wave 4, all hot-path heap use funnels through [`EvalScratch`]
//!    and the caller-owned output buffers, so a `no_std` port is a
//!    manifest/feature change (swapping `std` for `alloc` in
//!    [`EvalScratch`] and the row stores), not a kernel rewrite. The port
//!    itself is out of scope for v26.11.1.
//! 7. **`naive.rs` as a feature-gated file**: the spec's test-support
//!    reference unifier lives in this file's `#[cfg(test)]` module (the
//!    `reference-unifier` feature would require editing
//!    `Cargo.toml`/`lib.rs`, which are coordinator-owned). The differential
//!    court is unchanged — it runs the real naive recursive unifier as
//!    ground truth; only the file/feature packaging diverges.
//! 8. **`benches/branchy_baseline.rs`**: not created — a new file outside
//!    this lane's dispatch scope, and the BM0 ratio fence additionally
//!    needs hardware counters the court host must expose (receipted
//!    separately). Registered as BLOCKED scope, not silently dropped.
//!
//! Wave 9 (cross-lane integration, v26.11.1 wave 9): the scale court —
//! one million rows through the fixpoint driver + projector
//! (`stress_court_1m_rows_converges_chunk_stable_matches_closed_form_model`)
//! with an independent closed-form model as oracle, per-rule/per-round
//! popcount equality, derived-set equality, and M12-6 chunk-partition
//! stability AT SCALE (chunk sizes 262144 vs [`DEFAULT_CHUNK_ROWS`]). The
//! wave-9 emission-side seams (A04 frame identity, A05 power-cut WAL
//! record codec + per-offset sweep) live in `project.rs` where the
//! [`crate::project::DeltaBatch`] identity is minted.

use crate::project::{ConsequentProjector, DeltaBatch, DeltaRows};
use crate::sigil::{self, Sigil};
// W6-BN3NS no_std strip: alloc collections + core hash traits. The
// workspace carries no hashbrown, so alloc::collections is the
// dependency-free no_std form (justification in
// docs/jira/v26.11.2/waves/w6-bn3-nostd-borrowed-driver.md).
extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::hash::Hasher;

/// W5 deterministic hasher, retained as a frozen artifact only. The
/// current `seen` membership structure is the W6-BN3NS
/// `BTreeSet<(u64, u64, u64)>`; this struct is NOT in the hot loop. It
/// was once the static-seeded FxHash-style multiply-xor over the
/// `(u64, u64, u64)` triple, chosen because std `RandomState` carried a
/// TLS once-init `blr` into the hot loop (historical note:
/// waves/p2-asm-0blr-census.md, LBB68_43); after the BTreeSet move that
/// rationale no longer applies and the struct is retained solely as the
/// frozen seed artifact pinned to 0xBF58_476D_1CE4_E5B9 lineage.
#[allow(dead_code)] // W6-BN3NS: retained frozen seed artifact (0xBF58_476D_1CE4_E5B9); seen-set moved to BTreeSet
pub(crate) struct FixpointHasher(u64);

impl Default for FixpointHasher {
    fn default() -> Self {
        FixpointHasher(0xBF58_476D_1CE4_E5B9)
    }
}

impl Hasher for FixpointHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write(&mut self, _bytes: &[u8]) {
        // (u64,u64,u64) keys only ever arrive via write_u64 below; the
        // fallback stays correct (not a stub) for any other caller.
        for chunk in _bytes.chunks(8) {
            let mut buf = [0u8; 8];
            buf[..chunk.len()].copy_from_slice(chunk);
            let v = u64::from_le_bytes(buf);
            self.0 = (self.0 ^ v).wrapping_mul(0x517C_C1B7_2722_0A95);
        }
    }
    #[inline]
    fn write_u64(&mut self, v: u64) {
        self.0 = (self.0 ^ v).wrapping_mul(0x517C_C1B7_2722_0A95);
    }
}

// BuildHasherDefault<FixpointHasher>::default().build_hasher() yields
// FixpointHasher::default() = seeded state (0xBF58_476D_1CE4_E5B9).

/// Total width of the virtual register file: `R0..=R255`, the full 8-bit
/// register-number space the `Sigil::VarUni`/`VarExi` payloads transport.
/// (Spec 4.2; A11 Section 5.)
pub const REGISTER_COUNT: usize = 256;

/// Width of one virtual register: 8 bytes = one `u64` lane group.
/// (Spec 4.2: "256 virtual registers, each 8 bytes".)
pub const REGISTER_WIDTH_BYTES: usize = 8;

/// Total bytes of the register file: 256 x 8 = 2 KiB — an L1-resident
/// working set. (Arithmetic consequence of spec 4.2.)
pub const REGISTER_FILE_BYTES: usize = REGISTER_COUNT * REGISTER_WIDTH_BYTES;

/// Size of the constant-pool band `R0..=R63`: interned values loaded from
/// the rule's constant bindings at rule-compile time. (Spec 4.2.)
pub const CONST_POOL_REGS: usize = 64;

/// Size of the variable band `R64..=R191`: one register per distinct rule
/// variable. (Spec 4.2.)
pub const VAR_REGS: usize = 128;

/// Size of the scratch band `R192..=R255`: mask temporaries, join scatter
/// targets, consequent output staging. (Spec 4.2.)
pub const SCRATCH_REGS: usize = 64;

/// First register number of the variable band (`R64`). (Spec 4.2.)
pub const VAR_BAND_FIRST: usize = 64;

/// First register number of the scratch band (`R192`). (Spec 4.2.)
pub const SCRATCH_BAND_FIRST: usize = 192;

/// Maximum distinct variables a single rule may bind: the variable band
/// width. Exceeding this is the typed `UnifyError::RegisterOverflow`
/// (spec 4.2: "max 128 variables per rule; the corpus generator never
/// exceeds 8").
pub const MAX_RULE_VARIABLES: usize = VAR_REGS;

/// Default rows per atom-0 scan chunk (spec 4.3 step 1: batches arrive
/// chunked; the M12-6 court proves the partition does not change output).
pub const DEFAULT_CHUNK_ROWS: usize = 4096;

/// Default fixpoint round budget. A rule corpus that has not converged by
/// then is refused with `FixpointBudgetExceeded`, never spun forever.
pub const DEFAULT_MAX_ROUNDS: u32 = 64;

// --- ABI freeze: compile-time layout laws (spec 4.2) --------------------
// The Wave-1 gate's size/align pins. The struct/const definitions above are
// the frozen register ABI; these `const` asserts make any width, band-
// partition, or A11-transport drift a BUILD failure, not a court surprise
// (house pattern: `crate::sigil` lines 407-425,
// `crates/cgap-arrow/src/layout.rs`, `project.rs` lines 147-149). This
// block closes the gap this lane's Wave-1 slice names: the file's docstring
// claimed compile-time pins that were not on disk (grep `const _` = 0
// before this block).
//
// Anti-vacuity mutations (build MUST fail, E0080):
//   - widen `REGISTER_WIDTH_BYTES` 8 -> 16 (register-width + file-bytes
//     laws flip);
//   - shrink `VAR_REGS` 128 -> 127 (band-partition closure law flips).
const _: () = assert!(REGISTER_COUNT == 256);
// A11 transport width: the 8-bit VarUni/VarExi payload space this file's
// registers occupy (cite, not redefine — dispatch law).
const _: () = assert!(REGISTER_COUNT == crate::sigil::REGISTER_COUNT);
const _: () = assert!(REGISTER_WIDTH_BYTES == core::mem::size_of::<u64>());
const _: () = assert!(REGISTER_FILE_BYTES == REGISTER_COUNT * REGISTER_WIDTH_BYTES);
const _: () = assert!(REGISTER_FILE_BYTES == 2048);
const _: () = assert!(CONST_POOL_REGS == 64);
const _: () = assert!(VAR_REGS == 128);
const _: () = assert!(SCRATCH_REGS == 64);
// Band partition: contiguous, non-overlapping, exactly covering R0..=R255.
const _: () = assert!(CONST_POOL_REGS + VAR_REGS + SCRATCH_REGS == REGISTER_COUNT);
const _: () = assert!(VAR_BAND_FIRST == CONST_POOL_REGS);
const _: () = assert!(SCRATCH_BAND_FIRST == VAR_BAND_FIRST + VAR_REGS);
const _: () = assert!(SCRATCH_BAND_FIRST + SCRATCH_REGS == REGISTER_COUNT);
const _: () = assert!(MAX_RULE_VARIABLES == VAR_REGS);
const _: () = assert!(DEFAULT_CHUNK_ROWS > 0);
const _: () = assert!(DEFAULT_MAX_ROUNDS >= 1);
// Register-file size/align: 256 x 8 bytes, u64-aligned — the L1-resident
// working set of spec 4.2.
const _: () = assert!(core::mem::size_of::<RegisterFile>() == REGISTER_FILE_BYTES);
const _: () = assert!(core::mem::align_of::<RegisterFile>() == core::mem::size_of::<u64>());

/// Typed error surface of the unifier (spec 4.2). Wave 1 froze
/// `RegisterOverflow`; Wave 2 lands the kernel/join/digest error variants
/// the Wave-1 note deferred. Every boundary is a typed error, never a
/// lossy coercion (house rule, A11 spec Section 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnifyError {
    /// A rule body introduced more than [`MAX_RULE_VARIABLES`] (128)
    /// distinct variables — the variable band `R64..=R191` cannot hold
    /// the binding set. (Spec 4.2.)
    RegisterOverflow,
    /// A rule introduced more than [`CONST_POOL_REGS`] (64) distinct
    /// constants — the constant band `R0..=R63` cannot hold them.
    /// (Spec 4.2 band law.)
    ConstantPoolOverflow,
    /// `FormulaRef` in a rule's predicate slot — banned by A12 (A11
    /// `sigil.rs`: "Banned in the `p` slot by A12").
    FormulaRefInPredicate,
    /// A body/head term word carried a sigil the Wave-2 kernel does not
    /// unify: `VarExi` (existential scoping is a later semantic layer),
    /// `BuiltinOp` (A13's vocabulary), or a RESERVED code (A11: the
    /// decoder MUST refuse).
    UnsupportedTermSigil(u8),
    /// A `VarUni` payload exceeded the 8-bit register space `R0..R255`.
    RegisterOutOfRange(u64),
    /// A register access crossed its band (e.g. a constant store above
    /// `R63`). A compiled rule can only cause this through a compiler
    /// bug — surfaced as a typed error, never a silent wrap.
    RegisterBandViolation { reg: usize },
    /// A head template referenced a variable no body atom binds; the
    /// derivation would be undefined. (Typed refusal, spec 4.2.)
    UnboundHeadVariable,
    /// A rule with an empty body — nothing to select on.
    EmptyRuleBody,
    /// `chunk_size == 0` passed to the batch kernel.
    InvalidChunkSize,
    /// Statement columns of unequal length (malformed SoA input).
    ColumnLengthMismatch { s: usize, p: usize, o: usize },
    /// The fixpoint driver exceeded its round budget without converging.
    FixpointBudgetExceeded { rounds: u32 },
}

/// Term slot inside a body atom or head template. (Spec 4.1 s/p/o.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermSlot {
    /// Subject position.
    Subject,
    /// Predicate position.
    Predicate,
    /// Object position.
    Object,
}

impl TermSlot {
    fn index(self) -> usize {
        match self {
            TermSlot::Subject => 0,
            TermSlot::Predicate => 1,
            TermSlot::Object => 2,
        }
    }
}

const TERM_SLOTS: [TermSlot; 3] = [TermSlot::Subject, TermSlot::Predicate, TermSlot::Object];

/// A body atom: three raw A11 term words (subject, predicate, object).
/// Words are decoded and register-assigned at compile time; laws (p-slot
/// FormulaRef ban, sigil refusals, band widths) are enforced there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyAtom {
    /// Subject term word.
    pub s: u64,
    /// Predicate term word.
    pub p: u64,
    /// Object term word.
    pub o: u64,
}

impl BodyAtom {
    /// Build an atom from three raw term words.
    pub fn new(s: u64, p: u64, o: u64) -> Self {
        BodyAtom { s, p, o }
    }
}

/// An implication rule `body => head` (spec Section 1). Evaluation order
/// of `body` atoms is the corpus author's (spec 3.2: hand-ordered v1; the
/// differential court covers order-independence of *correctness*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Emitting-rule id carried into [`DeltaBatch::rule_id`].
    pub rule_id: u32,
    /// Body atoms, evaluated in order (nested-loop join, spec 3.2).
    pub body: Vec<BodyAtom>,
    /// Head template term words `(s, p, o)`.
    pub head: [u64; 3],
}

/// A body/head term after word decode and register assignment (spec 4.2):
/// `Const` holds a constant-pool register `R0..=R63`; `Var` indexes the
/// variable band (`R64 + idx`, `idx < 128`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledTerm {
    /// Constant pool register `R_n`, `n < CONST_POOL_REGS`.
    Const(u8),
    /// Variable band index: register `R{VAR_BAND_FIRST + idx}`.
    Var(u8),
}

/// A rule lowered to the register program (spec 4.2: "The op list is
/// emitted once per rule per batch-size class, not per row"). Constants
/// live in [`CompiledRule::const_pool`] (`R0..`), variables in the band
/// (`R64 + idx`); [`CompiledRule::var_first`] records each variable's
/// binding provenance `(atom index, term slot)` — the register-op
/// `BindVar` position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRule {
    /// Emitting-rule id.
    pub rule_id: u32,
    /// Body atoms in compiled form, evaluation order.
    pub atoms: Vec<[CompiledTerm; 3]>,
    /// Head template in compiled form.
    pub head: [CompiledTerm; 3],
    /// Constant pool `R0..Rk` (first-seen order over body, then head).
    pub const_pool: Vec<u64>,
    /// Distinct variable count (band width used).
    pub var_count: usize,
    /// Per variable index: the `(atom, slot)` that binds it (first
    /// occurrence, body scan order). `usize::MAX` atom = head-only
    /// variable (refused at compile time).
    pub var_first: Vec<(usize, usize)>,
}

/// SoA statement columns over A11 term words (recorded divergence 1: the
/// Arrow-batch form is the Wave-3 seam; the columnar contract — equal
/// lengths, position-aligned rows — is identical).
#[derive(Clone, Copy, Debug)]
pub struct StatementColumns<'a> {
    /// Subject column.
    pub s: &'a [u64],
    /// Predicate column.
    pub p: &'a [u64],
    /// Object column.
    pub o: &'a [u64],
}

impl<'a> StatementColumns<'a> {
    /// Build columns; refuses unequal lengths (typed, never silent).
    pub fn new(s: &'a [u64], p: &'a [u64], o: &'a [u64]) -> Result<Self, UnifyError> {
        if s.len() != p.len() || s.len() != o.len() {
            return Err(UnifyError::ColumnLengthMismatch {
                s: s.len(),
                p: p.len(),
                o: o.len(),
            });
        }
        Ok(StatementColumns { s, p, o })
    }

    /// Row count.
    pub fn len(&self) -> usize {
        self.s.len()
    }

    /// True when the store has no rows.
    pub fn is_empty(&self) -> bool {
        self.s.is_empty()
    }

    /// Column lookup by term-slot index. Out-of-range slots are LOUD
    /// (Wave-5 defect-D2 repair): the previous `_ => self.o` fallback arm
    /// silently served the object column to a miscomputed slot index —
    /// exactly the silent-wrap class mutation M12-3 / A20-audit-M10
    /// (`col(si)` -> `col(si + 1)`) exploits. Slot indices are
    /// compiler-produced (`TERM_SLOTS`, 0..=2); any other value is a
    /// caller bug and panics with a typed message instead of silently
    /// comparing against the wrong column. Courted by
    /// `columns_court_out_of_range_slot_is_loud`.
    fn col(&self, slot: usize) -> &[u64] {
        match slot {
            0 => self.s,
            1 => self.p,
            2 => self.o,
            other => unreachable!("StatementColumns::col: slot {other} out of range 0..=2"),
        }
    }
}

/// Borrowed/owned row source for the unify evaluation core (W6-BFX): a
/// read-only abstraction over "the store this round evaluates against".
/// [`StatementColumns`] is the plain borrowed store; [`SplitStore`] is the
/// fixpoint driver's borrowed-prefix + owned-delta-tail view, which lets
/// the driver evaluate over prefix and derived rows WITHOUT ever copying
/// the caller's input columns to the heap.
trait ColView {
    /// Row count.
    fn n(&self) -> usize;
    /// One word: slot `0..=2`, global row index `0..n()`. For
    /// [`StatementColumns`] this is exactly `col(slot)[row]`; for
    /// [`SplitStore`] rows `< prefix.len()` read the borrowed prefix,
    /// rows after it read the owned delta tail.
    fn w(&self, slot: usize, row: usize) -> u64;
}

impl ColView for StatementColumns<'_> {
    #[inline]
    fn n(&self) -> usize {
        self.len()
    }
    #[inline]
    fn w(&self, slot: usize, row: usize) -> u64 {
        self.col(slot)[row]
    }
}

/// Borrowed-prefix + owned-delta-tail store view (W6-BFX): the fixpoint
/// driver evaluates each round against prefix (the caller's columns,
/// never copied) followed by the delta tail (only derived rows, appended
/// in derivation order). Row order matches the retired full-store copy
/// (`prefix || delta` in append order), so match order, match sets, and
/// termination behavior are bit-identical.
struct SplitStore<'a> {
    prefix: &'a StatementColumns<'a>,
    ds: &'a [u64],
    dp: &'a [u64],
    do_: &'a [u64],
}

impl ColView for SplitStore<'_> {
    #[inline]
    fn n(&self) -> usize {
        self.prefix.len() + self.ds.len()
    }
    #[inline]
    fn w(&self, slot: usize, row: usize) -> u64 {
        let plen = self.prefix.len();
        if row < plen {
            self.prefix.col(slot)[row]
        } else {
            let t = row - plen;
            match slot {
                0 => self.ds[t],
                1 => self.dp[t],
                2 => self.do_[t],
                other => unreachable!("SplitStore::w: slot {other} out of range 0..=2"),
            }
        }
    }
}

/// The `R0..R255` register file (spec 4.2): scalar staging for constant
/// loads and per-match variable binding. Band accesses are checked — a
/// violation is the typed `UnifyError::RegisterBandViolation`, never a
/// silent wrap into a neighboring band.
#[derive(Clone, Debug)]
pub struct RegisterFile {
    regs: [u64; REGISTER_COUNT],
}

impl RegisterFile {
    /// A zeroed file (constants and variables overwrite their slots).
    pub fn zeroed() -> Self {
        RegisterFile {
            regs: [0; REGISTER_COUNT],
        }
    }

    /// `LoadConst`: store a rule constant into the constant band
    /// `R0..=R63`.
    pub fn load_const(&mut self, reg: usize, word: u64) -> Result<(), UnifyError> {
        if reg >= CONST_POOL_REGS {
            return Err(UnifyError::RegisterBandViolation { reg });
        }
        self.regs[reg] = word;
        Ok(())
    }

    /// `BindVar`: store a variable binding into `R{64 + idx}`,
    /// `idx < 128`.
    pub fn bind_var(&mut self, var_idx: usize, word: u64) -> Result<(), UnifyError> {
        let reg = VAR_BAND_FIRST + var_idx;
        if reg >= SCRATCH_BAND_FIRST {
            return Err(UnifyError::RegisterBandViolation { reg });
        }
        self.regs[reg] = word;
        Ok(())
    }

    /// Read a register.
    pub fn load(&self, reg: usize) -> Result<u64, UnifyError> {
        if reg >= REGISTER_COUNT {
            return Err(UnifyError::RegisterBandViolation { reg });
        }
        Ok(self.regs[reg])
    }
}

/// All full-body matches over one batch, aggregated across chunks (spec
/// 4.3 steps 2-4). Match `m` is: one row per body atom
/// ([`MatchSet::atom_rows`]) plus one bound value per variable
/// ([`MatchSet::var_values`]) — the selection vector and binding map the
/// projector consumes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MatchSet {
    /// Number of matches.
    pub len: usize,
    /// Per variable index (`R64 + idx`), the bound value for each match.
    pub var_values: Vec<Vec<u64>>,
    /// Per body atom, the matched store row for each match.
    pub atom_rows: Vec<Vec<u32>>,
}

/// The rule compiler (spec 5.2 C2: rule text -> register ops). Decodes A11
/// term words, interns constants into `R0..`, assigns variables into
/// `R64..`, records binding provenance, and enforces every band/ABI law as
/// a typed refusal.
#[derive(Clone, Copy, Debug, Default)]
pub struct RuleCompiler;

impl RuleCompiler {
    /// Compile a rule. Typed refusals: [`UnifyError::EmptyRuleBody`],
    /// [`UnifyError::FormulaRefInPredicate`],
    /// [`UnifyError::UnsupportedTermSigil`],
    /// [`UnifyError::RegisterOutOfRange`],
    /// [`UnifyError::ConstantPoolOverflow`],
    /// [`UnifyError::RegisterOverflow`],
    /// [`UnifyError::UnboundHeadVariable`].
    pub fn compile(rule: &Rule) -> Result<CompiledRule, UnifyError> {
        if rule.body.is_empty() {
            return Err(UnifyError::EmptyRuleBody);
        }
        let mut it = Interner::default();
        let mut atoms = Vec::with_capacity(rule.body.len());
        for (ai, atom) in rule.body.iter().enumerate() {
            let mut ct = [CompiledTerm::Const(0); 3];
            for (si, &word) in [atom.s, atom.p, atom.o].iter().enumerate() {
                ct[si] = it.intern(word, TERM_SLOTS[si], ai)?;
            }
            if it.const_pool.len() > CONST_POOL_REGS {
                return Err(UnifyError::ConstantPoolOverflow);
            }
            if it.vars.len() > MAX_RULE_VARIABLES {
                return Err(UnifyError::RegisterOverflow);
            }
            atoms.push(ct);
        }
        let mut head = [CompiledTerm::Const(0); 3];
        for (si, &word) in rule.head.iter().enumerate() {
            // Head terms bind nothing: atom index usize::MAX marks a
            // head-only variable.
            head[si] = it.intern(word, TERM_SLOTS[si], usize::MAX)?;
        }
        if it.const_pool.len() > CONST_POOL_REGS {
            return Err(UnifyError::ConstantPoolOverflow);
        }
        if it.vars.len() > MAX_RULE_VARIABLES {
            return Err(UnifyError::RegisterOverflow);
        }
        for term in &head {
            if let CompiledTerm::Var(idx) = *term {
                if it.var_first[idx as usize].0 == usize::MAX {
                    return Err(UnifyError::UnboundHeadVariable);
                }
            }
        }
        Ok(CompiledRule {
            rule_id: rule.rule_id,
            atoms,
            head,
            const_pool: it.const_pool,
            var_count: it.vars.len(),
            var_first: it.var_first,
        })
    }
}

#[derive(Default)]
struct Interner {
    const_pool: Vec<u64>,
    const_index: BTreeMap<u64, u8>,
    vars: Vec<u8>,
    var_index: BTreeMap<u8, u8>,
    var_first: Vec<(usize, usize)>,
}

impl Interner {
    fn intern(
        &mut self,
        word: u64,
        slot: TermSlot,
        atom_idx: usize,
    ) -> Result<CompiledTerm, UnifyError> {
        let sig = (word >> sigil::SIGIL_SHIFT) as u8;
        if sig == Sigil::VarUni as u8 {
            let payload = word & sigil::PAYLOAD_MASK;
            if payload >= sigil::REGISTER_COUNT as u64 {
                return Err(UnifyError::RegisterOutOfRange(payload));
            }
            let vp = payload as u8;
            let next = self.vars.len();
            let idx = *self.var_index.entry(vp).or_insert_with(|| {
                self.vars.push(vp);
                self.var_first.push((atom_idx, slot.index()));
                next as u8
            });
            return Ok(CompiledTerm::Var(idx));
        }
        if sig == Sigil::FormulaRef as u8 && slot == TermSlot::Predicate {
            return Err(UnifyError::FormulaRefInPredicate);
        }
        if sig == Sigil::SlotConst as u8 || sig == Sigil::FormulaRef as u8 {
            let next = self.const_pool.len();
            let reg = *self.const_index.entry(word).or_insert_with(|| {
                self.const_pool.push(word);
                next as u8
            });
            return Ok(CompiledTerm::Const(reg));
        }
        Err(UnifyError::UnsupportedTermSigil(sig))
    }
}

/// All-ones lane mask: the "match" element of the mask algebra.
const MASK_ALL: u64 = u64::MAX;

/// First slot of `atom` holding a variable term (the emit_join_masks
/// key lane; an all-const atom returns 0).
fn first_var_slot(atom: &[CompiledTerm; 3]) -> usize {
    (0..3)
        .find(|&si| matches!(atom[si], CompiledTerm::Var(_)))
        .unwrap_or(0)
}

/// Semi-join prefilter key-set cap: the mask arm pays one column pass
/// per distinct key, so it runs only when the distinct bound-key set is
/// at most this wide (otherwise the radix bucketing alone prunes).
const KEY_PREFILTER_MAX: usize = 4;

/// Branch-free u64-lane equality mask (the constant-time comparator arm;
/// spec ADR 3.3's honest scoping, doe-4 Section 2 "u64-lane arm"): all-ones
/// when `a == b`, zero otherwise. Pure mask arithmetic — XOR, neg, OR,
/// shift, subtract — so the executed instruction stream is identical for
/// every operand pair: no data-dependent branch, no early exit, no lookup.
/// Both comparator sites ([`atom_local_match_into`] and the join probe)
/// AND these masks into an accumulator and branch once on the final mask,
/// never per comparison.
///
/// Anti-vacuity anchor: the binding-alias comparator class (doe-4 Section 8
/// SHIM-DOE4) replaces the VALUE compare feeding this function; the
/// binding-consistency court and the differential court both flip under it
/// (receipt: `docs/jira/v26.11.1/waves/a12-finish-receipt.md`).
#[inline(always)]
fn lane_eq_mask(a: u64, b: u64) -> u64 {
    let d = a ^ b;
    // `nz` = 1 iff `d != 0`: for nonzero d, `d.wrapping_neg()` = `!d + 1`
    // has ones everywhere above d's highest set bit, so the OR sets bit 63
    // (and if d's highest set bit IS bit 63, d supplies it directly); zero
    // maps to zero. `nz.wrapping_sub(1)` then underflows to all-ones iff
    // equal.
    let nz = (d | d.wrapping_neg()) >> 63;
    nz.wrapping_sub(1)
}

/// Reusable per-chunk evaluation scratch (Wave 4 hardening, spec 7.4):
/// every heap buffer the nested-loop evaluation touches, owned once and
/// reused across chunks, fixpoint rounds, and court runs. The
/// zero-allocation hot path's substrate: after a warmup run sizes the
/// buffers on identical input, a second scan run performs zero
/// allocations (courted in the tests below; the mutation "reintroduce
/// per-call allocation" MUST flip that court).
#[derive(Debug, Default)]
pub struct EvalScratch {
    binds: Vec<(usize, u64)>,
    seen: Vec<(usize, usize)>,
    bound_flags: Vec<bool>,
    bound_vars: Vec<usize>,
    bound_slots: Vec<(usize, usize)>,
    cand_rows: Vec<u32>,
    cand_spans: Vec<(usize, usize)>,
    cand_binds: Vec<(usize, u64)>,
    cur_rows: Vec<Vec<u32>>,
    cur_vars: Vec<Vec<u64>>,
    nxt_rows: Vec<Vec<u32>>,
    nxt_vars: Vec<Vec<u64>>,
    /// Per-column capacity high-water marks (`rows` and `vars` sides).
    /// Updated at the end of every chunk from both buffer roles; used at
    /// chunk start to reserve BOTH sides of the cur/nxt double-buffer
    /// pair, so warm scratch keeps the expansion off the allocator no
    /// matter which role a column is serving.
    hw_rows: Vec<usize>,
    hw_vars: Vec<usize>,
}

/// L3 (W3 lever): caller-leased semi-join probe scratch. Every heap
/// buffer the radix probe touches, owned once and reused across chunks,
/// fixpoint rounds, and court runs — the same discipline as
/// [`EvalScratch`]. `Default` is empty buffers with `radix: true`
/// (the L3 fast path is the default; `radix: false` selects the retired
/// nested-loop probe, the M-L3-1 differential arm).
///
/// High-water marks mirror `EvalScratch::hw_rows`/`hw_vars`: the peak
/// bucket-table occupancy and mask width are recorded per chunk so a
/// caller can prewarm/reserve off the hot path.
#[derive(Debug)]
pub struct JoinProbeScratch {
    /// Radix probe enabled (default). `false` = nested-loop probe
    /// (the pre-L3 micro-architecture, kept as the M-L3-1 arm).
    pub radix: bool,
    /// Semi-join candidate survival mask, one bit per store row
    /// (populated only when the prefilter arm runs).
    pub mask_s: Vec<u64>,
    /// Per-bucket chain heads into the candidate list; exactly
    /// [`RADIX_BUCKETS`] (4096) entries, `u32::MAX` = empty bucket. The
    /// L1-resident scratchpad of the L3 design contract
    /// (13-arrow-star-schema.md): 4096 x 4 B = 16 KiB.
    pub bucket_head: Vec<u32>,
    /// Occupancy summary (BH7 XOR-filter pre-gate, repair-20): bit `b`
    /// of word `b/64` is set iff `bucket_head[b] != u32::MAX` (bucket
    /// non-empty), accumulated during `bucket_candidates` from the real
    /// chain build — so bit=0 ⇒ chain empty is EXACT, not a
    /// probabilistic fingerprint. The probe loop tests this single word
    /// bit before touching `bucket_head[b]`, skipping the head fetch +
    /// sentinel compare for empty buckets. NOTE: a byte-fingerprint
    /// summary (OR-fold of inserted key bytes) is UNSOUND for this gate
    /// and was rejected: OR is a union of byte SETS, so two distinct
    /// keys' union can contain a byte tuple no single key has — bit=1
    /// would not imply possible membership and bit=0 would not imply
    /// absence; only per-bucket occupancy read off the built chains is
    /// exact. 4096 buckets / 64 = 64 words, 512 B.
    pub bucket_occupancy: Vec<u64>,
    /// Singly-linked chain over candidate-list positions: `next_chain[c]`
    /// is the next candidate position in bucket `radix_bucket(key[c])`.
    /// Chains are built candidate-ascending (build walks positions in
    /// reverse, pushing each onto its bucket head), so emission order per
    /// bucket — and therefore match order — is identical to the nested
    /// loop.
    pub next_chain: Vec<u32>,
    /// Distinct bound-key words (sorted, deduped) for the semi-join
    /// prefilter arm.
    pub key_set: Vec<u64>,
    /// High-water mark: peak `next_chain` occupancy (mirrors
    /// `EvalScratch::hw_rows`).
    pub hw_bucket_rows: usize,
    /// High-water mark: peak `mask_s` word count.
    pub hw_mask_words: usize,
    /// Candidate-list cache (L3, W3 lever): the join-atom candidate list
    /// over the full store is CHUNK-INDEPENDENT (candidates come from the
    /// whole store, local constraints only), so within one
    /// [`unify_batch_probe`] call it is built once per join atom and
    /// reused across every chunk. The cache is invalidated at the start of
    /// every `unify_batch_probe` call (store contents may have changed
    /// between calls — the fixpoint driver appends rows each round), so
    /// soundness does not depend on the caller noticing store mutation.
    pub cand_rows_cache: Vec<u32>,
    pub cand_spans_cache: Vec<(usize, usize)>,
    pub cand_binds_cache: Vec<(usize, u64)>,
    /// Cache validity tag: atom/rule/store-length key (see
    /// `probe_cache_key`). `u64::MAX` = invalidated.
    pub cache_key: u64,
    pub cache_len: usize,
    /// Set by the caller to force the per-chunk candidate rebuild (the
    /// pre-L3 micro-architecture) even on the radix path — the M-L3-1
    /// differential/measurement arm.
    pub rebuild_per_chunk: bool,
    /// L3 receipt counters: candidate-cache hits (chunk/atom visits
    /// served from the cache) and misses (per-batch rebuilds) since
    /// lease.
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// L3 (W3 lever / FA3 radix-default flip): explicit `Default` so the
/// default probe matches the docstring — `radix: true` (the L3 fast
/// path). The retired nested-loop arm is selected explicitly
/// (`radix: false`), never by accident of a derived default. All other
/// field defaults are identical to the former `#[derive(Default)]`
/// (empty buffers, `u64::MAX` cache key, zero counters/marks).
impl Default for JoinProbeScratch {
    fn default() -> Self {
        Self {
            radix: true,
            mask_s: Vec::new(),
            bucket_head: Vec::new(),
            bucket_occupancy: Vec::new(),
            next_chain: Vec::new(),
            key_set: Vec::new(),
            hw_bucket_rows: 0,
            hw_mask_words: 0,
            cand_rows_cache: Vec::new(),
            cand_spans_cache: Vec::new(),
            cand_binds_cache: Vec::new(),
            cache_key: u64::MAX,
            cache_len: 0,
            rebuild_per_chunk: false,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

/// Radix bucket count: 2^12 = 4096 buckets — the L1-resident scratchpad
/// of the L3 design contract (`docs/jira/v26.11.2/13-arrow-star-schema.md`):
/// 4096 x 4 B bucket heads = 16 KiB, hash `h = (y ^ y >> 16) & 0x0FFF`
/// on the 56-bit payload window of the joined key word.
pub const RADIX_BUCKETS: usize = 4096;

/// Payload window of the point-range mask scan: the 56-bit term-word
/// payload (A11 transport width; `sigil::PAYLOAD_MASK`).
const PAYLOAD_WINDOW_MASK: u64 = 0x00FF_FFFF_FFFF_FFFF;

/// Low-4-payload-bit bucket of a key word (the radix hash of L3 point
/// 3). Payload-only so a sigil-bit difference can never mis-bucket a
/// row out of its bucket — the inner probe still compares full words.
#[inline(always)]
fn radix_bucket(word: u64) -> usize {
    let y = word & PAYLOAD_WINDOW_MASK;
    ((y ^ (y >> 16)) & 0x0FFF) as usize
}

/// L3 cache key: FNV-1a over (rule identity, atom index, atom terms,
/// store length). The cache is invalidated at every
/// [`unify_batch_probe`] entry, so the key only has to be unique within
/// one batch evaluation — rule identity fields + atom index + store
/// length are sufficient there.
fn probe_cache_key(rule: &CompiledRule, ai: usize, n: usize) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut mix = |x: u64| {
        h ^= x;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    };
    mix(rule.rule_id as u64);
    mix(rule.atoms.len() as u64);
    mix(rule.var_count as u64);
    mix(ai as u64);
    mix(n as u64);
    for term in &rule.atoms[ai] {
        let v = match *term {
            CompiledTerm::Const(c) => 0x8000_0000 | c as u64,
            CompiledTerm::Var(v) => 0x4000_0000 | v as u64,
        };
        mix(v);
    }
    h
}

/// Build the per-bucket singly-linked chains over the candidate list
/// `rows` (the L3 design-contract shape: `bucket_head` +
/// `next_chain`). Walking candidate positions in REVERSE and pushing each
/// onto its bucket head makes every chain ascending in candidate
/// position, so emission order per bucket — and therefore match order —
/// is identical to the nested loop.
fn bucket_candidates(
    head: &mut Vec<u32>,
    next: &mut Vec<u32>,
    occ: &mut Vec<u64>,
    key_at: impl Fn(usize) -> u64,
    rows: &[u32],
) {
    head.clear();
    head.resize(RADIX_BUCKETS, u32::MAX);
    occ.clear();
    occ.resize(RADIX_BUCKETS.div_ceil(64), 0);
    next.clear();
    next.resize(rows.len(), 0);
    for c in (0..rows.len()).rev() {
        let b = radix_bucket(key_at(rows[c] as usize));
        next[c] = head[b];
        head[b] = c as u32;
        occ[b >> 6] |= 1u64 << (b & 63);
    }
}

/// Mask-AND semi-join composition over per-row-group validity masks —
/// VENDORED from `crates/cgap-parquet/src/bitslice.rs:348`
/// (`semi_join_masks`, verbatim algebra) so cgap-bn3 carries no
/// cgap-parquet dependency (levers L3 point 2). Bit `t` of `out[g]` is
/// set iff row `64*g + t` qualified on BOTH mask arms. Returns the
/// qualifying-row popcount so callers assert final state.
fn semi_join_masks(a: &[u64], b: &[u64], out: &mut [u64]) -> u64 {
    assert!(
        a.len() == b.len(),
        "semi_join_masks: mask length mismatch {} vs {}",
        a.len(),
        b.len()
    );
    assert!(
        out.len() >= a.len(),
        "semi_join_masks: out.len()={} < {}",
        out.len(),
        a.len()
    );
    let mut rows = 0u64;
    for g in 0..a.len() {
        let m = a[g] & b[g];
        out[g] = m;
        rows += m.count_ones() as u64;
    }
    rows
}

/// Point-range bit-slice semi-join mask emission (levers L3 point 2):
/// for each 64-row group, bit `t` of `out_masks[g]` has row `64*g + t`
/// set iff the row survives BOTH mask arms — (a) the join key lane's
/// 56-bit payload equals one of the distinct `bound_keys` (a point
/// range `lo == hi == key`, the degenerate case of the
/// `scan_range_vertical` bit-slice scan semantics, w = 56 payload
/// window), AND (b) every `Const` slot of `join_atom` matches the row
/// exactly (full word). The two arms compose through the vendored
/// semi_join_masks AND-algebra. Returns the qualifying-row popcount.
///
/// Contract: `out_masks.len() >= ceil(cols.len()/64)` (loud assert).
/// Superset prefilter: the inner probe still compares full words, so
/// the differential courts pin identical match sets with and without
/// the mask arm.
pub fn emit_join_masks(
    cols: &StatementColumns<'_>,
    join_atom: &[CompiledTerm; 3],
    const_pool: &[u64],
    bound_keys: &[u64],
    out_masks: &mut [u64],
) -> u64 {
    emit_join_masks_view(cols, join_atom, const_pool, bound_keys, out_masks)
}

/// [`emit_join_masks`] over any [`ColView`] (W6-BFX): the fixpoint driver's
/// split store uses the same point-range mask algebra over borrowed-prefix
/// + delta-tail rows. Identical masks bit-for-bit (same row order).
fn emit_join_masks_view<V: ColView>(
    cols: &V,
    join_atom: &[CompiledTerm; 3],
    const_pool: &[u64],
    bound_keys: &[u64],
    out_masks: &mut [u64],
) -> u64 {
    let n = cols.n();
    let groups = n.div_ceil(64);
    assert!(
        out_masks.len() >= groups,
        "emit_join_masks: out_masks.len()={} < groups={groups}",
        out_masks.len()
    );
    let const_slots: Vec<usize> = (0..3)
        .filter(|&si| matches!(join_atom[si], CompiledTerm::Const(_)))
        .collect();
    let first_var_slot = (0..3)
        .find(|&si| matches!(join_atom[si], CompiledTerm::Var(_)))
        .unwrap_or(0);
    let mut rows = 0u64;
    for g in 0..groups {
        let lo = g * 64;
        let hi = (lo + 64).min(n);
        let mut m_key = 0u64;
        let mut m_const = 0u64;
        for t in lo..hi {
            let w = cols.w(first_var_slot, t) & PAYLOAD_WINDOW_MASK;
            let mut hit = false;
            for &k in bound_keys {
                hit |= lane_eq_mask(w, k & PAYLOAD_WINDOW_MASK) != 0;
            }
            m_key |= (hit as u64) << (t - lo);
            let mut mc = MASK_ALL;
            for &si in &const_slots {
                if let CompiledTerm::Const(creg) = join_atom[si] {
                    mc &= lane_eq_mask(cols.w(si, t), const_pool[creg as usize]);
                }
            }
            m_const |= ((mc == MASK_ALL) as u64) << (t - lo);
        }
        let mut semi_out = [0u64; 1];
        rows += semi_join_masks(&[m_key], &[m_const], &mut semi_out);
        out_masks[g] = semi_out[0];
    }
    rows
}

/// Per-atom local constraints over one candidate row: constant equality
/// and within-atom variable repetition (the self-join `?x p ?x`). Slots
/// whose variable is already bound (`bound[idx]`) are skipped here — the
/// per-parent probe loop checks them against the match's binding. On
/// success the fresh bindings this row contributes are written to
/// `binds` (cleared first); returns `false` if the row fails any slot.
/// Allocation-free after warmup: `binds`/`seen` are caller-owned scratch.
fn atom_local_match_into<V: ColView>(
    const_pool: &[u64],
    atom: &[CompiledTerm; 3],
    cols: &V,
    row: usize,
    bound: &[bool],
    binds: &mut Vec<(usize, u64)>,
    seen: &mut Vec<(usize, usize)>,
) -> bool {
    binds.clear();
    seen.clear();
    // Constant-time comparator arm: the slot comparisons AND their
    // [`lane_eq_mask`]s into `acc` with no data-dependent early exit — the
    // conjunction of pure comparisons is order-independent, so accept/reject
    // semantics and first-occurrence binding order are identical to the
    // retired early-return form. One branch, on the final mask.
    let mut acc = MASK_ALL;
    for si in 0..3 {
        let w = cols.w(si, row);
        match atom[si] {
            CompiledTerm::Const(creg) => {
                acc &= lane_eq_mask(w, const_pool[creg as usize]);
            }
            CompiledTerm::Var(idx) => {
                let idx = idx as usize;
                if bound[idx] {
                    continue;
                }
                match seen.iter().copied().find(|&(_, fi)| fi == idx) {
                    Some((fs, _)) => {
                        acc &= lane_eq_mask(w, cols.w(fs, row));
                    }
                    None => {
                        seen.push((si, idx));
                        binds.push((idx, w));
                    }
                }
            }
        }
    }
    acc == MASK_ALL
}

/// Evaluate one compiled rule over one batch (all body atoms), atom-0
/// scan chunked over `[0, cols.len())`. Join atoms (1..) probe the FULL
/// column set, so a match pair is never split by a chunk boundary — this
/// is the chunk-partition independence the M12-6 determinism court pins:
/// match order is (atom-0 row ascending, join candidate ascending)
/// regardless of partitioning (recorded divergence 3 for the
/// micro-architecture).
///
/// Allocation profile (Wave 4): with a warm [`EvalScratch`] and a
/// `MatchSet` retaining capacity from a prior run over identical input,
/// this performs zero heap allocations (court `zero_alloc_court_...`).
pub fn unify_batch(
    rule: &CompiledRule,
    cols: &StatementColumns<'_>,
    chunk_size: usize,
) -> Result<MatchSet, UnifyError> {
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    let mut probe = JoinProbeScratch::default();
    unify_batch_probe(rule, cols, chunk_size, &mut scratch, &mut out, &mut probe)?;
    Ok(out)
}

/// [`unify_batch`] over caller-owned buffers (Wave 4, spec 7.4): reuses
/// `scratch` across chunks and appends into `out` (cleared first,
/// capacity retained) — the zero-allocation hot path's entry point.
///
/// L3 (W3): the join probe runs on a fresh default [`JoinProbeScratch`]
/// (radix semi-join). The leased form is [`unify_batch_probe`]; this
/// signature is preserved verbatim so the Wave-4 zero-alloc callers
/// (benches, courts) are API-stable.
pub fn unify_batch_into(
    rule: &CompiledRule,
    cols: &StatementColumns<'_>,
    chunk_size: usize,
    scratch: &mut EvalScratch,
    out: &mut MatchSet,
) -> Result<(), UnifyError> {
    let mut probe = JoinProbeScratch::default();
    unify_batch_probe(rule, cols, chunk_size, scratch, out, &mut probe)
}

/// [`unify_batch_into`] over a caller-leased [`JoinProbeScratch`] (L3,
/// levers spec): the radix bucket tables + semi-join mask buffers are
/// owned once and reused across chunks/rounds/court runs, mirroring the
/// [`EvalScratch`] allocation discipline. `probe.radix == false`
/// selects the retired nested-loop probe — the M-L3-1 differential arm.
pub fn unify_batch_probe(
    rule: &CompiledRule,
    cols: &StatementColumns<'_>,
    chunk_size: usize,
    scratch: &mut EvalScratch,
    out: &mut MatchSet,
    probe: &mut JoinProbeScratch,
) -> Result<(), UnifyError> {
    unify_batch_probe_view(rule, cols, chunk_size, scratch, out, probe)
}

/// [`unify_batch_probe`] over any [`ColView`] (W6-BFX): the concrete pub
/// signature is preserved verbatim for the zero-alloc callers; the
/// fixpoint driver reaches the split store through this generic core.
/// Identical chunking, probe order, and match semantics — the view is
/// read-only, and `StatementColumns`' `ColView` impl is exactly
/// `col(slot)[row]` (zero-cost monomorphization on the concrete path).
fn unify_batch_probe_view<V: ColView>(
    rule: &CompiledRule,
    cols: &V,
    chunk_size: usize,
    scratch: &mut EvalScratch,
    out: &mut MatchSet,
    probe: &mut JoinProbeScratch,
) -> Result<(), UnifyError> {
    if chunk_size == 0 {
        return Err(UnifyError::InvalidChunkSize);
    }
    // Output columns sized to the rule; `resize` grows only on first use,
    // so a reused `MatchSet` keeps its high-water capacity.
    if out.atom_rows.len() < rule.atoms.len() {
        out.atom_rows.resize(rule.atoms.len(), Vec::new());
    }
    if out.var_values.len() < rule.var_count {
        out.var_values.resize(rule.var_count, Vec::new());
    }
    out.len = 0;
    // L3 cache invalidation: store contents may have changed since the
    // last call over this lease (the fixpoint driver appends every
    // round), so the candidate cache never survives a batch boundary.
    probe.cache_key = u64::MAX;
    probe.cache_len = 0;
    for col in out.atom_rows.iter_mut() {
        col.clear();
    }
    for col in out.var_values.iter_mut() {
        col.clear();
    }
    let n = cols.n();
    let mut start = 0usize;
    while start < n {
        let end = (start + chunk_size).min(n);
        eval_chunk_into(rule, cols, start, end, scratch, out, probe);
        start = end;
    }
    debug_assert!(out
        .var_values
        .iter()
        .take(rule.var_count)
        .all(|v| v.len() == out.atom_rows[0].len()));
    out.len = out.atom_rows[0].len();
    Ok(())
}

/// One chunk of the nested-loop evaluation (Wave-2 algorithm, Wave-4
/// buffers): identical match semantics and match order to the Wave-2
/// `eval_chunk`, with every heap buffer drawn from [`EvalScratch`].
fn eval_chunk_into<V: ColView>(
    rule: &CompiledRule,
    cols: &V,
    start: usize,
    end: usize,
    sc: &mut EvalScratch,
    out: &mut MatchSet,
    probe: &mut JoinProbeScratch,
) {
    // Chunk-local match state, REBUILT per atom expansion: every accepted
    // join pair pushes exactly one entry into every variable column (the
    // parent's value for already-bound vars, the candidate's value for
    // fresh vars), so column alignment holds by construction. The output
    // accumulators (`out`) only see the final, aligned chunk state.
    for col in sc.cur_rows.iter_mut() {
        col.clear();
    }
    for col in sc.cur_vars.iter_mut() {
        col.clear();
    }
    if sc.cur_rows.len() < rule.atoms.len() {
        sc.cur_rows.resize(rule.atoms.len(), Vec::new());
    }
    if sc.cur_vars.len() < rule.var_count {
        sc.cur_vars.resize(rule.var_count, Vec::new());
    }
    if sc.nxt_rows.len() < rule.atoms.len() {
        sc.nxt_rows.resize(rule.atoms.len(), Vec::new());
    }
    if sc.nxt_vars.len() < rule.var_count {
        sc.nxt_vars.resize(rule.var_count, Vec::new());
    }
    // Capacity high-water: keep BOTH roles of the cur/nxt pair warm (see
    // `EvalScratch::hw_rows`). The reserve is a no-op once satisfied —
    // after warmup it allocates nothing; on the first runs it sizes the
    // expansion-side buffers, which the double-buffer swap alone would
    // leave cold every second pass.
    sc.hw_rows.resize(rule.atoms.len(), 0);
    sc.hw_vars.resize(rule.var_count, 0);
    for a in 0..rule.atoms.len() {
        sc.cur_rows[a].reserve(sc.hw_rows[a]);
        sc.nxt_rows[a].reserve(sc.hw_rows[a]);
    }
    for idx in 0..rule.var_count {
        sc.cur_vars[idx].reserve(sc.hw_vars[idx]);
        sc.nxt_vars[idx].reserve(sc.hw_vars[idx]);
    }

    // Atom 0: candidates are this chunk's rows (spec 4.3 step 2); no
    // variable is bound yet.
    sc.bound_flags.clear();
    sc.bound_flags.resize(rule.var_count, false);
    for i in start..end {
        if atom_local_match_into(
            &rule.const_pool,
            &rule.atoms[0],
            cols,
            i,
            &sc.bound_flags,
            &mut sc.binds,
            &mut sc.seen,
        ) {
            sc.cur_rows[0].push(i as u32);
            for (idx, val) in sc.binds.iter().copied() {
                sc.cur_vars[idx].push(val);
            }
        }
    }

    // Atoms 1..: semi-join expansion (spec 3.2). Candidates come from the
    // full store; bound variables constrain each candidate against the
    // match's bindings; fresh variables bind from the candidate row.
    for (ai, atom) in rule.atoms.iter().enumerate().skip(1) {
        // Vars with first binding before this atom are bound for every
        // current match; the rest are fresh here. Scratch columns are
        // cleared FIRST (they carry the previous atom's state).
        sc.bound_flags.clear();
        sc.bound_vars.clear();
        sc.bound_flags.resize(rule.var_count, false);
        for (idx, &(atom_idx, _)) in rule.var_first.iter().enumerate() {
            sc.bound_flags[idx] = atom_idx < ai;
            if atom_idx < ai {
                sc.bound_vars.push(idx);
            }
        }
        // Bound slots: (slot, var index) pairs the join probe checks —
        // computed before the candidate build because the L3 cache gate
        // reads it. An atom with NO pre-bound variable carries no join
        // semantics to fast-path: it falls to the nested arm, which owns
        // `sc.cand_*`.
        sc.bound_slots.clear();
        for (si, term) in atom.iter().enumerate() {
            if let CompiledTerm::Var(idx) = *term {
                if sc.bound_flags[idx as usize] {
                    sc.bound_slots.push((si, idx as usize));
                }
            }
        }
        // L3 candidate cache: the candidate list over the FULL store is
        // chunk-independent (candidates are local constraints only), so
        // on the cached arm it is built once per atom per batch into the
        // probe's cache fields and reused across every chunk — the
        // pre-L3 code rebuilt it per chunk (a 262144-chunk batch over
        // 1M rows paid 256 full-column scans per batch). The nested arm
        // (`radix == false`) and the `rebuild_per_chunk` arm keep the
        // per-chunk rebuild in `sc.cand_*` (the M-L3-1 measurement arm).
        let use_cache = probe.radix && !probe.rebuild_per_chunk && !sc.bound_slots.is_empty();
        let ck = probe_cache_key(rule, ai, cols.n());
        let mut rebuilt_cache = false;
        if use_cache && probe.cache_key != ck {
            probe.cand_rows_cache.clear();
            probe.cand_spans_cache.clear();
            probe.cand_binds_cache.clear();
            for j in 0..cols.n() {
                if atom_local_match_into(
                    &rule.const_pool,
                    atom,
                    cols,
                    j,
                    &sc.bound_flags,
                    &mut sc.binds,
                    &mut sc.seen,
                ) {
                    probe
                        .cand_spans_cache
                        .push((probe.cand_binds_cache.len(), sc.binds.len()));
                    probe.cand_binds_cache.extend_from_slice(&sc.binds);
                    probe.cand_rows_cache.push(j as u32);
                }
            }
            probe.cache_key = ck;
            probe.cache_len = probe.cand_rows_cache.len();
            probe.cache_misses += 1;
            rebuilt_cache = true;
        } else if use_cache {
            probe.cache_hits += 1;
        }
        if !use_cache {
            sc.cand_rows.clear();
            sc.cand_spans.clear();
            sc.cand_binds.clear();
            for j in 0..cols.n() {
                if atom_local_match_into(
                    &rule.const_pool,
                    atom,
                    cols,
                    j,
                    &sc.bound_flags,
                    &mut sc.binds,
                    &mut sc.seen,
                ) {
                    sc.cand_spans.push((sc.cand_binds.len(), sc.binds.len()));
                    sc.cand_binds.extend_from_slice(&sc.binds);
                    sc.cand_rows.push(j as u32);
                }
            }
        }
        let cur = sc.cur_rows[0].len();
        // Double-buffer swap: `nxt` carries the previous atom's state
        // (cleared for use below), `cur` the current expansion's; the
        // swap moves buffer ownership without touching the allocator.
        for col in sc.nxt_rows.iter_mut().take(ai + 1) {
            col.clear();
        }
        for col in sc.nxt_vars.iter_mut() {
            col.clear();
        }
        if probe.radix && !sc.bound_slots.is_empty() {
            // L3 radix semi-join probe (levers L3 point 3): bucket the
            // candidate list once per atom by the low 4 payload bits of
            // the FIRST bound slot's candidate word, then each match
            // probes only its own bucket. Bucketing is a superset
            // filter (full-word equality implies payload-equality), so
            // the surviving candidate set per match — and the emission
            // order (candidates visited in ascending index order within
            // each bucket) — is identical to the nested loop.
            let key_slot = sc.bound_slots[0].0;
            // W6-BFX: the key "column" is an indexed view over the
            // ColView (split-store rows read prefix or delta by global
            // index); bucket_candidates takes the closure directly.
            let key_at = |g: usize| cols.w(key_slot, g);
            // Optional semi-join prefilter arm (levers L3 point 2): a
            // single-bound-slot join whose distinct bound-key set is
            // tiny (<= [`KEY_PREFILTER_MAX`]) pre-masks the candidate
            // scan before bucketing via [`emit_join_masks`] (the
            // per-row survival mask over the point-range key set).
            let single =
                sc.bound_slots.len() == 1 && sc.bound_slots[0].0 == first_var_slot(&rule.atoms[ai]);
            probe.key_set.clear();
            let vidx0 = sc.bound_slots[0].1;
            let cur_col = &sc.cur_vars[vidx0];
            for &v in cur_col.iter().take(cur) {
                probe.key_set.push(v);
            }
            probe.key_set.sort_unstable();
            probe.key_set.dedup();
            let prefilter = single && probe.key_set.len() <= KEY_PREFILTER_MAX;
            probe.mask_s.clear();
            if prefilter {
                probe.mask_s.resize(cols.n().div_ceil(64), 0);
                let _ = emit_join_masks_view(
                    cols,
                    &rule.atoms[ai],
                    &rule.const_pool,
                    &probe.key_set,
                    &mut probe.mask_s,
                );
            }
            // Bucket build: stable partition of the candidate list by
            // low-4-payload-bit bucket; counts precomputed, then one
            // prefix-sum pass gives (start, end) group ranges. L3: the
            // bucket partition is chunk-independent (candidates and the
            // key column are chunk-independent), so on the cached arm it
            // rides the cache rebuild; the rebuild arm re-buckets the
            // per-chunk list every chunk.
            if rebuilt_cache {
                bucket_candidates(
                    &mut probe.bucket_head,
                    &mut probe.next_chain,
                    &mut probe.bucket_occupancy,
                    key_at,
                    &probe.cand_rows_cache,
                );
            } else if !use_cache {
                bucket_candidates(
                    &mut probe.bucket_head,
                    &mut probe.next_chain,
                    &mut probe.bucket_occupancy,
                    key_at,
                    &sc.cand_rows,
                );
            }
            probe.hw_bucket_rows = probe.hw_bucket_rows.max(probe.next_chain.len());
            // Candidate sources for the probe loop: the cached list
            // (L3 fast path) or the per-chunk rebuilt list.
            let (cand_rows, cand_spans, cand_binds) = if use_cache {
                (
                    &probe.cand_rows_cache,
                    &probe.cand_spans_cache,
                    &probe.cand_binds_cache,
                )
            } else {
                (&sc.cand_rows, &sc.cand_spans, &sc.cand_binds)
            };
            // Probe loop: O(cur + cand + hits). Per match, visit only
            // the match's key bucket (emission order preserved: within
            // a bucket, candidate indices are ascending).
            for m in 0..cur {
                let kv = sc.cur_vars[sc.bound_slots[0].1][m];
                let b = radix_bucket(kv);
                // Repair-20 (BH7 XOR-filter pre-gate): exact occupancy
                // bit first — bit=0 ⇒ chain provably empty, skip the
                // bucket_head fetch + sentinel compare entirely.
                if probe.bucket_occupancy[b >> 6] >> (b & 63) & 1 == 0 {
                    continue;
                }
                let mut c = probe.bucket_head[b];
                while c != u32::MAX {
                    let j = cand_rows[c as usize];
                    if prefilter && probe.mask_s[j as usize / 64] >> (j % 64) & 1 == 0 {
                        c = probe.next_chain[c as usize];
                        continue;
                    }
                    let (b0, blen) = cand_spans[c as usize];
                    let mut ok = MASK_ALL;
                    for &(si, vidx) in &sc.bound_slots {
                        ok &= lane_eq_mask(cols.w(si, j as usize), sc.cur_vars[vidx][m]);
                    }
                    if ok != MASK_ALL {
                        c = probe.next_chain[c as usize];
                        continue;
                    }
                    for a in 0..ai {
                        sc.nxt_rows[a].push(sc.cur_rows[a][m]);
                    }
                    sc.nxt_rows[ai].push(j);
                    for &vidx in &sc.bound_vars {
                        let v = sc.cur_vars[vidx][m];
                        sc.nxt_vars[vidx].push(v);
                    }
                    for &(idx, val) in &cand_binds[b0..b0 + blen] {
                        sc.nxt_vars[idx].push(val);
                    }
                    c = probe.next_chain[c as usize];
                }
            }
        } else {
            for m in 0..cur {
                for (c, &j) in sc.cand_rows.iter().enumerate() {
                    let (b0, blen) = sc.cand_spans[c];
                    let mut ok = MASK_ALL;
                    for &(si, vidx) in &sc.bound_slots {
                        ok &= lane_eq_mask(cols.w(si, j as usize), sc.cur_vars[vidx][m]);
                    }
                    if ok != MASK_ALL {
                        continue;
                    }
                    for a in 0..ai {
                        sc.nxt_rows[a].push(sc.cur_rows[a][m]);
                    }
                    sc.nxt_rows[ai].push(j);
                    for &vidx in &sc.bound_vars {
                        let v = sc.cur_vars[vidx][m];
                        sc.nxt_vars[vidx].push(v);
                    }
                    for &(idx, val) in &sc.cand_binds[b0..b0 + blen] {
                        sc.nxt_vars[idx].push(val);
                    }
                }
            }
        }
        core::mem::swap(&mut sc.cur_rows, &mut sc.nxt_rows);
        core::mem::swap(&mut sc.cur_vars, &mut sc.nxt_vars);
    }

    // Append the aligned chunk state into the shared accumulators. The
    // scratch may hold extra columns from a previous, wider rule (the
    // driver shares one scratch across rules) — bound every loop to THIS
    // rule's widths, never the scratch's.
    for (a, column) in sc.cur_rows.iter_mut().enumerate().take(rule.atoms.len()) {
        out.atom_rows[a].extend_from_slice(column);
    }
    for (idx, column) in sc.cur_vars.iter_mut().enumerate().take(rule.var_count) {
        out.var_values[idx].extend_from_slice(column);
    }

    // Record the chunk's per-column high-water usage from both buffer
    // roles (post-final-swap `cur` holds the matches, `nxt` the previous
    // atom's state — between them they cover every expansion peak).
    for a in 0..rule.atoms.len() {
        let peak = sc.cur_rows[a].len().max(sc.nxt_rows[a].len());
        sc.hw_rows[a] = sc.hw_rows[a].max(peak);
    }
    for idx in 0..rule.var_count {
        let peak = sc.cur_vars[idx].len().max(sc.nxt_vars[idx].len());
        sc.hw_vars[idx] = sc.hw_vars[idx].max(peak);
    }
}

/// Evaluate a rule set to fixpoint (spec 7.2: "run the register unifier to
/// fixpoint"). Each round evaluates every rule over the store-so-far,
/// emits one [`DeltaBatch`] per rule via the projector, and appends newly
/// derived rows to the store. The `seen` set is termination machinery of
/// the driver only — the projector never dedups (M12-7 division), and
/// batches re-emit re-derivations for A15 to collapse.
///
/// Epochs are the round numbers (monotonically increasing, spec 4.4).
/// Deterministic: rule order as given, match order per [`unify_batch`].
pub fn unify_to_fixpoint(
    rules: &[Rule],
    cols: &StatementColumns<'_>,
) -> Result<Vec<DeltaBatch<DeltaRows>>, UnifyError> {
    unify_to_fixpoint_opt(rules, cols, DEFAULT_MAX_ROUNDS, DEFAULT_CHUNK_ROWS)
}

/// [`unify_to_fixpoint`] with explicit round budget and atom-0 chunk size
/// (the M12-6 court sweeps the chunk size through this entry point).
pub fn unify_to_fixpoint_opt(
    rules: &[Rule],
    cols: &StatementColumns<'_>,
    max_rounds: u32,
    chunk_size: usize,
) -> Result<Vec<DeltaBatch<DeltaRows>>, UnifyError> {
    let compiled: Vec<CompiledRule> = rules
        .iter()
        .map(RuleCompiler::compile)
        .collect::<Result<Vec<_>, _>>()?;
    // W6-BFX (borrowed-prefix + owned-delta-tail): the caller's columns
    // are NEVER copied to the heap. Derived rows append to small owned
    // delta buffers only; each round evaluates over `prefix || delta` in
    // append order — the identical row sequence the retired full-store
    // copy presented, so match order, derived-fact sets, and termination
    // behavior (`seen` is membership-only) are unchanged.
    let mut ds: Vec<u64> = Vec::new();
    let mut dp: Vec<u64> = Vec::new();
    let mut do_: Vec<u64> = Vec::new();
    // Termination machinery only (see fn docs): membership, never order.
    // W6-BN3NS: BTreeSet<(u64, u64, u64)> (D5-verified membership-only) —
    // no hasher at all, so no RandomState TLS and no hash calls in the
    // hot loop; ordered B-tree gives the same insert/contains semantics.
    let mut seen: BTreeSet<(u64, u64, u64)> = (0..cols.len())
        .map(|i| (cols.s[i], cols.p[i], cols.o[i]))
        .collect();

    let projector = ConsequentProjector::new();
    // Wave 4: one scratch + one match set, reused across rounds x rules
    // (allocation discipline; match semantics unchanged).
    let mut scratch = EvalScratch::default();
    let mut matches = MatchSet::default();
    // W6-BFX: one JoinProbeScratch hoisted to driver scope and leased
    // through `unify_batch_probe_view` for every rule x round — the radix
    // bucket tables + semi-join mask buffers are owned once, never
    // re-allocated per rule per round (the retired
    // `unify_batch_into` arm allocated a fresh one at each call).
    let mut probe = JoinProbeScratch::default();
    let mut batches = Vec::new();
    let mut epoch: u32 = 0;
    loop {
        epoch += 1;
        let mut new_rows = 0usize;
        for rule in &compiled {
            // Round view: borrowed prefix (caller's columns, zero-copy)
            // plus the owned delta tail. `SplitStore` borrows the delta
            // buffers; pushes below extend them, so the view is rebuilt
            // per rule (never held across a push).
            {
                let store = SplitStore {
                    prefix: cols,
                    ds: &ds,
                    dp: &dp,
                    do_: &do_,
                };
                unify_batch_probe_view(
                    rule,
                    &store,
                    chunk_size,
                    &mut scratch,
                    &mut matches,
                    &mut probe,
                )?;
            }
            let batch = projector.project(rule, epoch as u64, &matches)?;
            for i in 0..batch.rows.len() {
                let t = (batch.rows.s[i], batch.rows.p[i], batch.rows.o[i]);
                if seen.insert(t) {
                    ds.push(t.0);
                    dp.push(t.1);
                    do_.push(t.2);
                    new_rows += 1;
                }
            }
            batches.push(batch);
        }
        if new_rows == 0 {
            return Ok(batches);
        }
        if epoch >= max_rounds {
            return Err(UnifyError::FixpointBudgetExceeded { rounds: epoch });
        }
    }
}

#[cfg(test)]
// [W2-9 unblock 2026-11-01, disclosed per fan-out SLA] `pub(crate)` so the
// noalloc.rs courts can read the SAME counting-allocator thread-local via
// `current_allocs` — a second `#[global_allocator]` is E0152, and the
// binary's single Counting allocator must stay the one counting substrate.
pub(crate) mod tests {
    use super::*;

    // --- Wave-1 ABI courts (kept verbatim) --------------------------------

    /// Band-partition court (spec 4.2): the three bands are contiguous,
    /// non-overlapping, and exactly cover `R0..=R255`.
    /// Mutation that MUST flip this court: shrink `VAR_REGS` 128 -> 127
    /// (the const asserts above fail the build first; this runtime court
    /// is the redundant receipt).
    #[test]
    fn band_partition_court_bands_cover_the_file() {
        assert_eq!(CONST_POOL_REGS, 64);
        assert_eq!(VAR_REGS, 128);
        assert_eq!(SCRATCH_REGS, 64);
        assert_eq!(VAR_BAND_FIRST, 64);
        assert_eq!(SCRATCH_BAND_FIRST, 192);
        assert_eq!(CONST_POOL_REGS + VAR_REGS + SCRATCH_REGS, REGISTER_COUNT);
        // Contiguity: each band starts where the previous band ends, and
        // the scratch band closes the file at R255.
        assert_eq!(SCRATCH_BAND_FIRST + SCRATCH_REGS, REGISTER_COUNT - 1 + 1);
        // Every register number the 8-bit payload can transport falls in
        // exactly one band.
        for n in 0u64..REGISTER_COUNT as u64 {
            let in_const = n < CONST_POOL_REGS as u64;
            let in_var = (VAR_BAND_FIRST as u64..SCRATCH_BAND_FIRST as u64).contains(&n);
            let in_scratch = n >= SCRATCH_BAND_FIRST as u64;
            assert!(
                (in_const ^ in_var ^ in_scratch) && !(in_const && in_var && in_scratch),
                "R{n} banded {in_const}/{in_var}/{in_scratch}",
            );
        }
    }

    /// Register-width court (spec 4.2): one register is one `u64` lane
    /// group; the whole file is 2 KiB.
    /// Mutation that MUST flip this court: widen `REGISTER_WIDTH_BYTES`
    /// 8 -> 16 (the `size_of::<u64>()` const law fails the build first).
    #[test]
    fn register_width_court_file_is_2048_bytes() {
        assert_eq!(REGISTER_WIDTH_BYTES, core::mem::size_of::<u64>());
        assert_eq!(REGISTER_FILE_BYTES, 2048);
    }

    /// Typed-error court (spec 4.2): `RegisterOverflow` is the named
    /// refusal for a rule exceeding the variable band.
    /// Mutation that MUST flip this court: rename the variant or drop it
    /// from the enum (build failure).
    #[test]
    fn typed_error_court_register_overflow_is_named() {
        let err = UnifyError::RegisterOverflow;
        assert_eq!(err, UnifyError::RegisterOverflow);
        assert!(format!("{err:?}").contains("RegisterOverflow"));
    }

    // --- Test substrate: term-word builders, corpus, naive oracle --------
    // (Self-contained: the oracle re-implements word decoding and rule
    // matching row-at-a-time with a hash map, a genuinely different
    // algorithm from the kernel — the differential court's independence.)

    const SIG_VAR: u64 = 0x01;

    /// SlotConstant term word (sigil 0x00): word == interned id.
    fn cw(id: u64) -> u64 {
        id
    }
    /// VarUni term word: register number in the payload.
    fn vw(reg: u8) -> u64 {
        (SIG_VAR << 56) | reg as u64
    }

    struct Corpus {
        s: Vec<u64>,
        p: Vec<u64>,
        o: Vec<u64>,
        rules: Vec<Rule>,
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state >> 33
    }

    /// Rule classes of spec 7.1: single-match, join, self-join,
    /// constant-bound, head-rebinds-swapped. Predicates: P1=100, P2=200,
    /// P3=300; node pool ids 1..=6.
    fn rule_set() -> Vec<Rule> {
        vec![
            // 0: single-match, same-order rebind
            Rule {
                rule_id: 0,
                body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
                head: [vw(10), cw(300), vw(11)],
            },
            // 1: join ?x P1 ?y . ?y P2 ?z => ?z P1 ?x
            Rule {
                rule_id: 1,
                body: vec![
                    BodyAtom::new(vw(10), cw(100), vw(11)),
                    BodyAtom::new(vw(11), cw(200), vw(12)),
                ],
                head: [vw(12), cw(100), vw(10)],
            },
            // 2: self-join ?x P2 ?x => ?x P1 (const 1)
            Rule {
                rule_id: 2,
                body: vec![BodyAtom::new(vw(10), cw(200), vw(10))],
                head: [vw(10), cw(100), cw(1)],
            },
            // 3: constant-bound (1 P1 ?y) => (?y P2 1)
            Rule {
                rule_id: 3,
                body: vec![BodyAtom::new(cw(1), cw(100), vw(11))],
                head: [vw(11), cw(200), cw(1)],
            },
            // 4: head re-binds body variables in swapped order
            Rule {
                rule_id: 4,
                body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
                head: [vw(11), cw(300), vw(10)],
            },
        ]
    }

    /// Deterministic (seeded) corpus: 24 rows, self-loop rows every 5th
    /// (feeds the self-join rule). Same seed -> same corpus (asserted by
    /// `corpus_court_same_seed_is_identical`).
    fn gen_corpus(seed: u64) -> Corpus {
        let mut st = seed | 1;
        let mut rows: Vec<(u64, u64, u64)> = Vec::new();
        for r in 0..24u64 {
            let subj = cw(1 + lcg(&mut st) % 6);
            if r % 5 == 0 {
                rows.push((subj, cw(200), subj));
            } else {
                let pred = cw(100 + (lcg(&mut st) % 2) * 100);
                let obj = cw(1 + lcg(&mut st) % 6);
                rows.push((subj, pred, obj));
            }
        }
        Corpus {
            s: rows.iter().map(|r| r.0).collect(),
            p: rows.iter().map(|r| r.1).collect(),
            o: rows.iter().map(|r| r.2).collect(),
            rules: rule_set(),
        }
    }

    /// Naive row-at-a-time matcher: recursive nested loops over row
    /// indices with a hash-map binding — the ground-truth algorithm of the
    /// differential court (spec 7.2). Decodes term words independently of
    /// the kernel.
    fn naive_match_rule(
        rule: &Rule,
        s: &[u64],
        p: &[u64],
        o: &[u64],
        binding: &mut BTreeMap<u8, u64>,
        out: &mut Vec<(u64, u64, u64)>,
    ) {
        fn resolve(w: u64, binding: &BTreeMap<u8, u64>) -> Option<u64> {
            if w >> 56 == SIG_VAR {
                binding.get(&((w & sigil::PAYLOAD_MASK) as u8)).copied()
            } else {
                Some(w)
            }
        }
        fn go(
            rule: &Rule,
            ai: usize,
            s: &[u64],
            p: &[u64],
            o: &[u64],
            binding: &mut BTreeMap<u8, u64>,
            out: &mut Vec<(u64, u64, u64)>,
        ) {
            if ai == rule.body.len() {
                let mut h = [0u64; 3];
                for (si, &w) in rule.head.iter().enumerate() {
                    match resolve(w, binding) {
                        Some(v) => h[si] = v,
                        // Unbound head variable: not derivable here (the
                        // kernel refuses such rules at compile time; the
                        // witnessed corpora contain none).
                        None => return,
                    }
                }
                out.push((h[0], h[1], h[2]));
                return;
            }
            let atom = &rule.body[ai];
            let slots = [atom.s, atom.p, atom.o];
            let columns: [&[u64]; 3] = [s, p, o];
            for j in 0..s.len() {
                let saved = binding.clone();
                let mut ok = true;
                for si in 0..3 {
                    let want = slots[si];
                    let have = columns[si][j];
                    if want >> 56 == SIG_VAR {
                        let key = (want & sigil::PAYLOAD_MASK) as u8;
                        match binding.get(&key) {
                            Some(&v) if v == have => {}
                            Some(_) => {
                                ok = false;
                                break;
                            }
                            None => {
                                binding.insert(key, have);
                            }
                        }
                    } else if want != have {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    go(rule, ai + 1, s, p, o, binding, out);
                }
                *binding = saved;
            }
        }
        go(rule, 0, s, p, o, binding, out);
    }

    /// Naive fixpoint oracle: rebuilds full-closure column views per round
    /// so the matcher sees derived rows too (genuinely row-at-a-time over
    /// the whole closure, independent of the kernel's store handling).
    /// Returns (full closure set, per-rule fired witness) — the witness is
    /// the "every rule fires" property of spec DoD 7 / M12-5.
    fn naive_closure_full(
        rules: &[Rule],
        s: &[u64],
        p: &[u64],
        o: &[u64],
    ) -> (BTreeSet<(u64, u64, u64)>, Vec<bool>) {
        let mut closure: BTreeSet<(u64, u64, u64)> =
            (0..s.len()).map(|i| (s[i], p[i], o[i])).collect();
        let mut fired = vec![false; rules.len()];
        loop {
            let cs: Vec<(u64, u64, u64)> = closure.iter().copied().collect();
            let vs: Vec<u64> = cs.iter().map(|r| r.0).collect();
            let vp: Vec<u64> = cs.iter().map(|r| r.1).collect();
            let vo: Vec<u64> = cs.iter().map(|r| r.2).collect();
            let mut grew = false;
            for (ri, rule) in rules.iter().enumerate() {
                let mut out = Vec::new();
                let mut binding = BTreeMap::new();
                naive_match_rule(rule, &vs, &vp, &vo, &mut binding, &mut out);
                for row in out {
                    if closure.insert(row) {
                        fired[ri] = true;
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
        (closure, fired)
    }

    /// Generator with the "every rule fires" witness (spec DoD 7): the
    /// corpus is retried deterministically until every rule derives at
    /// least one statement; otherwise M12-5 (rule drop) would be vacuous.
    fn gen_witnessed_corpus(seed: u64) -> Corpus {
        for k in 0..200u64 {
            let seed_k = seed.wrapping_mul(7919).wrapping_add(k);
            let c = gen_corpus(seed_k);
            let (_, fired) = naive_closure_full(&c.rules, &c.s, &c.p, &c.o);
            if fired.iter().all(|&f| f) {
                return c;
            }
        }
        panic!("corpus generator: no witnessed corpus within 200 retries for seed {seed}");
    }

    // --- Compile-law courts ----------------------------------------------

    /// Register-assignment court (spec 4.2): constants intern into R0..
    /// in first-seen order; variables map into the band `R64 + idx`;
    /// body and head share registers for the same variable.
    /// Mutation that MUST flip this court: assigning variables from R0
    /// (band mix) or per-occurrence registers (no sharing).
    #[test]
    fn compile_court_register_band_assignment() {
        let rule = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
            head: [vw(11), cw(300), vw(10)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        assert_eq!(c.const_pool, vec![cw(100), cw(300)]);
        assert_eq!(c.var_count, 2);
        // var 10 -> idx 0 (R64), var 11 -> idx 1 (R65); provenance both
        // from atom 0 (slots 0 and 2).
        assert_eq!(c.var_first, vec![(0, 0), (0, 2)]);
        assert_eq!(
            c.atoms[0],
            [
                CompiledTerm::Var(0),
                CompiledTerm::Const(0),
                CompiledTerm::Var(1)
            ]
        );
        // Head shares the body's registers: Var(1), Const(1), Var(0).
        assert_eq!(
            c.head,
            [
                CompiledTerm::Var(1),
                CompiledTerm::Const(1),
                CompiledTerm::Var(0)
            ]
        );
    }

    /// RegisterOverflow court (spec 4.2): a rule with 129 distinct
    /// variables is refused with the typed variant.
    /// Mutation that MUST flip this court: raising the band past the
    /// payload's 8-bit space or silently truncating register numbers.
    #[test]
    fn compile_court_register_overflow_is_typed() {
        let mut body = Vec::new();
        for v in 0..129u8 {
            body.push(BodyAtom::new(vw(v), cw(100), vw(v)));
        }
        let rule = Rule {
            rule_id: 1,
            body,
            head: [vw(0), cw(100), vw(0)],
        };
        assert_eq!(
            RuleCompiler::compile(&rule),
            Err(UnifyError::RegisterOverflow)
        );
    }

    /// ConstantPoolOverflow court (band law, spec 4.2): 65 distinct
    /// constants exceed `R0..=R63`.
    /// Mutation that MUST flip this court: wrapping constant registers
    /// modulo 64.
    #[test]
    fn compile_court_constant_pool_overflow_is_typed() {
        let mut body = Vec::new();
        for k in 0..65u64 {
            body.push(BodyAtom::new(cw(1000 + k), cw(100), cw(2000 + k)));
        }
        let rule = Rule {
            rule_id: 1,
            body,
            head: [cw(1000), cw(100), cw(2000)],
        };
        assert_eq!(
            RuleCompiler::compile(&rule),
            Err(UnifyError::ConstantPoolOverflow)
        );
    }

    /// Predicate-slot court (A12 law, A11 sigil.rs note): FormulaRef in
    /// the p slot is refused; in s/o slots it is an ordinary constant
    /// word (word equality).
    /// Mutation that MUST flip this court: allowing FormulaRef in p.
    #[test]
    fn compile_court_formula_ref_banned_in_predicate() {
        let fref = (0x03u64 << 56) | 7;
        let bad = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(vw(10), fref, vw(11))],
            head: [vw(10), cw(100), vw(11)],
        };
        assert_eq!(
            RuleCompiler::compile(&bad),
            Err(UnifyError::FormulaRefInPredicate)
        );
        let ok = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(fref, cw(100), vw(11))],
            head: [vw(11), cw(100), fref],
        };
        assert!(RuleCompiler::compile(&ok).is_ok());
    }

    /// Sigil-refusal court (A11: RESERVED must be refused; A12 Wave 2
    /// refuses VarExi and BuiltinOp loudly rather than guessing scoping).
    /// Mutation that MUST flip this court: treating unknown sigils as
    /// constants (silent widening).
    #[test]
    fn compile_court_unsupported_sigils_are_typed_refusals() {
        let cases = [
            ((0x02u64 << 56) | 5, UnifyError::UnsupportedTermSigil(0x02)), // VarExi
            ((0x04u64 << 56) | 9, UnifyError::UnsupportedTermSigil(0x04)), // BuiltinOp
            ((0xFFu64 << 56) | 1, UnifyError::UnsupportedTermSigil(0xFF)), // RESERVED
        ];
        for (word, expect) in cases {
            let rule = Rule {
                rule_id: 1,
                body: vec![BodyAtom::new(word, cw(100), vw(11))],
                head: [vw(11), cw(100), vw(11)],
            };
            assert_eq!(RuleCompiler::compile(&rule), Err(expect));
        }
    }

    /// Register-range court: a VarUni payload beyond R255 is refused.
    /// Mutation that MUST flip this court: truncating payloads to 8 bits.
    #[test]
    fn compile_court_register_out_of_range_is_typed() {
        let rule = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(vw(10), cw(100), (0x01u64 << 56) | 300)],
            head: [vw(10), cw(100), vw(10)],
        };
        assert_eq!(
            RuleCompiler::compile(&rule),
            Err(UnifyError::RegisterOutOfRange(300))
        );
    }

    /// Head-binding court: a head variable no body atom binds is refused.
    /// Mutation that MUST flip this court: instantiating unbound head
    /// variables as wildcard/zero.
    #[test]
    fn compile_court_unbound_head_variable_is_typed() {
        let rule = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
            head: [vw(12), cw(100), vw(10)],
        };
        assert_eq!(
            RuleCompiler::compile(&rule),
            Err(UnifyError::UnboundHeadVariable)
        );
    }

    /// Empty-body court.
    #[test]
    fn compile_court_empty_body_is_typed() {
        let rule = Rule {
            rule_id: 1,
            body: vec![],
            head: [cw(1), cw(100), cw(2)],
        };
        assert_eq!(RuleCompiler::compile(&rule), Err(UnifyError::EmptyRuleBody));
    }

    /// Register-file band court (spec 4.2): constant stores above R63,
    /// variable binds past R191, and reads past R255 are typed
    /// violations.
    /// Mutation that MUST flip this court: removing a band check.
    #[test]
    fn register_file_court_band_violations_are_typed() {
        let mut rf = RegisterFile::zeroed();
        assert_eq!(
            rf.load_const(CONST_POOL_REGS, 1),
            Err(UnifyError::RegisterBandViolation {
                reg: CONST_POOL_REGS
            })
        );
        assert_eq!(
            rf.bind_var(VAR_REGS, 1),
            Err(UnifyError::RegisterBandViolation {
                reg: SCRATCH_BAND_FIRST
            })
        );
        assert_eq!(
            rf.load(REGISTER_COUNT),
            Err(UnifyError::RegisterBandViolation {
                reg: REGISTER_COUNT
            })
        );
        // Lawful accesses round-trip.
        rf.load_const(0, 42).unwrap();
        rf.bind_var(0, 7).unwrap();
        assert_eq!(rf.load(0).unwrap(), 42);
        assert_eq!(rf.load(VAR_BAND_FIRST).unwrap(), 7);
    }

    // --- Kernel courts ----------------------------------------------------

    /// Single-atom selection court: const/var mask arithmetic selects
    /// exactly the rows satisfying the atom, with bindings materialized.
    /// Mutation that MUST flip this court (spec 7.3 M12-1 class):
    /// combining the three per-slot masks with OR instead of AND.
    #[test]
    fn kernel_court_single_atom_selection_and_bindings() {
        let s = [cw(1), cw(2), cw(1)];
        let p = [cw(100), cw(100), cw(200)];
        let o = [cw(3), cw(4), cw(5)];
        let rule = Rule {
            rule_id: 0,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
            head: [vw(11), cw(300), vw(10)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let m = unify_batch(&c, &cols, 2).unwrap(); // chunked on purpose
        assert_eq!(m.len, 2);
        assert_eq!(m.atom_rows[0], vec![0, 1]);
        assert_eq!(m.var_values[0], vec![cw(1), cw(2)], "?x bindings");
        assert_eq!(m.var_values[1], vec![cw(3), cw(4)], "?y bindings");
    }

    /// Self-join court (`?x p ?x`): within-atom variable repetition
    /// compares both slots' columns row-wise.
    /// Mutation that MUST flip this court (spec 7.3 M12-3 class):
    /// swapping the comparison direction of the repeated variable.
    #[test]
    fn kernel_court_self_join_within_atom() {
        let s = [cw(1), cw(1), cw(3)];
        let p = [cw(200), cw(200), cw(200)];
        let o = [cw(1), cw(2), cw(3)];
        let rule = Rule {
            rule_id: 2,
            body: vec![BodyAtom::new(vw(10), cw(200), vw(10))],
            head: [vw(10), cw(100), cw(1)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let m = unify_batch(&c, &cols, 4096).unwrap();
        assert_eq!(m.len, 2);
        assert_eq!(m.atom_rows[0], vec![0, 2]);
    }

    /// Occurs/self-join semantics court (doe-4 Section 7 INV-2): within-
    /// atom variable repetition (`?x p ?x`) is WITHIN-ROW value equality on
    /// the row's own slot columns; the differential oracle's occurs-check
    /// semantics (M12-8) are the preserved reference, so the kernel's
    /// derivations must equal the naive oracle's on an adversarial store —
    /// rows whose values are swapped ACROSS rows (8,9 then 9,8) must not
    /// self-join: no row has subject == object.
    /// Mutation that MUST flip this court: any comparator that matches a
    /// repeated variable across rows, or drops the within-row check.
    #[test]
    fn semantics_court_occurs_self_join_is_within_row_and_matches_oracle() {
        // 0: (7,P3,7) MATCH; 1: (8,P3,9) no; 2: (9,P3,8) no (cross-row swap);
        // 3: (5,P3,5) MATCH.
        let s = [cw(7), cw(8), cw(9), cw(5)];
        let p = [cw(300); 4];
        let o = [cw(7), cw(9), cw(8), cw(5)];
        let rule = Rule {
            rule_id: 8,
            body: vec![BodyAtom::new(vw(10), cw(300), vw(10))],
            head: [vw(10), cw(100), cw(1)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let m = unify_batch(&c, &cols, 4096).unwrap();
        assert_eq!(m.len, 2);
        assert_eq!(m.atom_rows[0], vec![0, 3]);
        assert_eq!(m.var_values[0], vec![cw(7), cw(5)]);
        // Occurs-check reference: the naive oracle's independent matcher
        // derives exactly the same two rows, same order (M12-8 semantics).
        let mut out = Vec::new();
        let mut binding = BTreeMap::new();
        naive_match_rule(&rule, &s, &p, &o, &mut binding, &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], (cw(7), cw(100), cw(1)));
        assert_eq!(out[1], (cw(5), cw(100), cw(1)));
        // Typed-refusal half of INV-2: an unbound head variable stays a
        // named compile refusal (compile court co-witnesses this arm).
        let bad = Rule {
            rule_id: 9,
            body: vec![BodyAtom::new(vw(10), cw(300), cw(7))],
            head: [vw(99), cw(100), cw(1)],
        };
        assert_eq!(
            RuleCompiler::compile(&bad),
            Err(UnifyError::UnboundHeadVariable)
        );
    }

    /// Cross-row join court: `?x P1 ?y . ?y P2 ?z` over a hand-computed
    /// store, including a candidate that passes atom-1 locally but fails
    /// the bound-variable probe.
    /// Mutation that MUST flip this court (spec 7.3 M12-3): swapping the
    /// join's bound-value probe direction (probing the binding against
    /// the candidate column instead of the candidate against bindings)
    /// scrambles at least one match here.
    #[test]
    fn kernel_court_cross_row_join() {
        // 0: (A,P1,B)  1: (B,P2,C)  2: (A,P1,D)  3: (D,P2,E)
        // 4: (B,P1,C)  5: (C,P2,F)  6: (A,P1,A)  <- y=A has no P2 out
        let s = [cw(1), cw(2), cw(1), cw(4), cw(2), cw(3), cw(1)];
        let p = [
            cw(100),
            cw(200),
            cw(100),
            cw(200),
            cw(100),
            cw(200),
            cw(100),
        ];
        let o = [cw(2), cw(3), cw(4), cw(5), cw(3), cw(6), cw(1)];
        let rule = Rule {
            rule_id: 1,
            body: vec![
                BodyAtom::new(vw(10), cw(100), vw(11)),
                BodyAtom::new(vw(11), cw(200), vw(12)),
            ],
            head: [vw(12), cw(100), vw(10)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let m = unify_batch(&c, &cols, 3).unwrap();
        assert_eq!(m.len, 3);
        // (x=A,y=B,z=C) (x=A,y=D,z=E) (x=B,y=C,z=F)
        assert_eq!(m.atom_rows[0], vec![0, 2, 4], "atom-0 anchors");
        assert_eq!(m.atom_rows[1], vec![1, 3, 5], "joined rows");
        assert_eq!(m.var_values[2], vec![cw(3), cw(5), cw(6)], "?z bindings");
    }

    /// Binding-consistency court (doe-4 Section 7 INV-1): one variable
    /// binds exactly one value per match, and the bound-slot probe compares
    /// the match's bound VALUE (register-dereferenced) — never the
    /// variable's register number or slot position.
    /// Mutation that MUST flip this court (doe-4 Section 8 SHIM-DOE4, run
    /// in a /tmp sandbox this wave): an aliasing comparator comparing the
    /// register NUMBER `R{64+idx}` against the candidate lane — the planted
    /// raw word 65 (= ?y's register number) then falsely joins trap row 2,
    /// and the exact-match/exact-binding asserts fail.
    #[test]
    fn semantics_court_binding_consistency_value_probe_not_alias() {
        // Rule ?x P1 ?y . ?y P2 ?z: ?x->idx0/R64, ?y->idx1/R65, ?z->idx2/R66.
        let rule = Rule {
            rule_id: 7,
            body: vec![
                BodyAtom::new(vw(10), cw(100), vw(11)),
                BodyAtom::new(vw(11), cw(200), vw(12)),
            ],
            head: [vw(12), cw(100), vw(10)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        assert_eq!(c.var_count, 3);
        let reg_of = |idx: usize| (VAR_BAND_FIRST + idx) as u64;
        assert_eq!(reg_of(1), 65);
        // Rows: 0 (1,P1,2) real anchor; 1 (2,P2,3) real join; 2 (65,P2,9)
        // TRAP — the subject IS the raw register number 65, so a value-blind
        // comparator aliases the ?y binding onto it; 3 (2,P2,5) real join.
        let s = [cw(1), cw(2), 65, cw(2)];
        let p = [cw(100), cw(200), cw(200), cw(200)];
        let o = [cw(2), cw(3), cw(9), cw(5)];
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let m = unify_batch(&c, &cols, 2).unwrap();
        // Exactly one bound value per variable per match (column alignment).
        for col in m.var_values.iter().take(c.var_count) {
            assert_eq!(col.len(), m.len);
        }
        // Exact matches under the value probe: rows 1 and 3 join; trap row 2
        // is rejected because the bound ?y VALUE cw(2) != its subject word 65.
        assert_eq!(m.len, 2, "alias trap row must not join");
        assert_eq!(m.atom_rows[0], vec![0, 0]);
        assert_eq!(m.atom_rows[1], vec![1, 3]);
        // Binding consistency: each variable's value is the store word at
        // its first-binding atom's matched row, for every match.
        for (idx, &(fa, fs)) in c.var_first.iter().enumerate() {
            for mm in 0..m.len {
                let anchor = m.atom_rows[fa][mm] as usize;
                assert_eq!(
                    m.var_values[idx][mm],
                    cols.col(fs)[anchor],
                    "var {idx} match {mm}"
                );
            }
        }
        assert_eq!(m.var_values[0], vec![cw(1), cw(1)]);
        assert_eq!(m.var_values[1], vec![cw(2), cw(2)]);
        assert_eq!(m.var_values[2], vec![cw(3), cw(5)]);
    }

    /// No-match court: a constant bound to an absent value selects
    /// nothing and yields a well-formed empty match set.
    /// Mutation that MUST flip this court: emitting phantom matches.
    #[test]
    fn kernel_court_no_match_is_empty() {
        let s = [cw(1)];
        let p = [cw(100)];
        let o = [cw(2)];
        let rule = Rule {
            rule_id: 0,
            body: vec![BodyAtom::new(cw(999), cw(100), vw(10))],
            head: [vw(10), cw(200), cw(1)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let m = unify_batch(&c, &cols, 4096).unwrap();
        assert_eq!(m.len, 0);
        assert!(m.atom_rows.iter().all(|v| v.is_empty()));
        assert_eq!(m.var_values[0], Vec::<u64>::new());
    }

    /// Chunk-size court: `chunk_size == 0` is a typed refusal.
    #[test]
    fn kernel_court_zero_chunk_size_is_typed() {
        let s = [cw(1)];
        let p = [cw(100)];
        let o = [cw(2)];
        let rule = Rule {
            rule_id: 0,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
            head: [vw(10), cw(100), vw(11)],
        };
        let c = RuleCompiler::compile(&rule).unwrap();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        assert_eq!(unify_batch(&c, &cols, 0), Err(UnifyError::InvalidChunkSize));
    }

    /// Column-law court: unequal columns are a typed refusal.
    #[test]
    fn columns_court_length_mismatch_is_typed() {
        let s = [cw(1), cw(2)];
        let p = [cw(100)];
        let o = [cw(2)];
        assert!(StatementColumns::new(&s, &p, &o).is_err());
    }

    /// Out-of-range slot court (Wave-5 defect-D2 repair): `col` refuses a
    /// slot index beyond the three term slots LOUDLY — never a silent
    /// fallback to the object column (the silent-wrap class mutation
    /// M12-3 / A20-audit-M10 exploits).
    /// Mutation that MUST flip this court: restoring the `_ => self.o`
    /// fallback arm — no panic is raised, `should_panic` FAILS
    /// (shim-court W5-M10 re-witnesses this flip on the repaired file).
    #[test]
    #[should_panic(expected = "out of range")]
    fn columns_court_out_of_range_slot_is_loud() {
        let s = [cw(1)];
        let p = [cw(100)];
        let o = [cw(2)];
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let _ = cols.col(3);
    }

    // --- Driver courts -----------------------------------------------------

    /// Fixpoint termination court: a self-deriving rule (`?x P1 ?x =>
    /// ?x P1 ?x` over `(A,P1,A)`) converges after one round — the driver
    /// cannot spin forever on a fixed point, and re-derivations are
    /// re-emitted (the projector emits derivations; dedup is A15's).
    /// Mutation that MUST flip this court: looping forever (budget hang)
    /// or deduping in the projector (popcount drops to 0).
    #[test]
    fn driver_court_self_derivation_terminates_and_reemits() {
        let s = [cw(1)];
        let p = [cw(100)];
        let o = [cw(1)];
        let rule = Rule {
            rule_id: 7,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(10))],
            head: [vw(10), cw(100), vw(10)],
        };
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let batches = unify_to_fixpoint(&[rule], &cols).unwrap();
        assert_eq!(batches.len(), 1, "one round, one rule");
        let b = &batches[0];
        assert_eq!(b.rule_id, 7);
        assert_eq!(b.epoch, 1);
        assert_eq!(b.source_mask_popcount, 1);
        assert_eq!(b.rows.len(), 1, "re-derivation emitted, not collapsed");
        assert_eq!(b.rows.row(0), (cw(1), cw(100), cw(1), 0));
    }

    /// Budget court: `max_rounds` is enforced with the typed variant.
    /// Mutation that MUST flip this court: ignoring the budget (unbounded
    /// loop) or returning Ok past the budget.
    #[test]
    fn driver_court_round_budget_is_typed() {
        let s = [cw(1)];
        let p = [cw(100)];
        let o = [cw(2)];
        let rule = Rule {
            rule_id: 0,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
            head: [vw(10), cw(200), vw(11)],
        };
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        // The rule derives a fresh row in round 1; a budget of 1 must
        // refuse before any second round.
        assert_eq!(
            unify_to_fixpoint_opt(&[rule], &cols, 1, 4096),
            Err(UnifyError::FixpointBudgetExceeded { rounds: 1 })
        );
    }

    /// Corpus determinism court (spec 7.1): the same seed generates the
    /// identical corpus.
    #[test]
    fn corpus_court_same_seed_is_identical() {
        for seed in 0..10u64 {
            let a = gen_corpus(seed);
            let b = gen_corpus(seed);
            assert_eq!(a.s, b.s);
            assert_eq!(a.p, b.p);
            assert_eq!(a.o, b.o);
            assert_eq!(a.rules, b.rules);
        }
    }

    /// Differential court (spec 7.2, the lane's anti-silent-wrongness
    /// anchor): over 100 witnessed (corpus, batch) pairs, the register
    /// unifier's fixpoint derivations equal the naive recursive
    /// unifier's — and every rule's id appears in the emission (the
    /// driver-side M12-5 witness; dropping a rule is observable).
    /// Mutations that MUST flip this court: M12-1 (mask AND -> OR),
    /// M12-3 (join probe swap), M12-5 (silent rule drop), M12-8 (oracle
    /// occurs-check mutation — the court's own positive control).
    #[test]
    fn differential_court_matches_naive_oracle_over_100_witnessed_corpora() {
        for seed in 0..100u64 {
            let c = gen_witnessed_corpus(seed);
            let cols = StatementColumns::new(&c.s, &c.p, &c.o).unwrap();
            // Two chunkings: default and one-row chunks (forces every
            // join probe across a chunk boundary — recorded divergence 1
            // property: chunking never changes derivations).
            for chunk in [DEFAULT_CHUNK_ROWS, 1] {
                let batches =
                    unify_to_fixpoint_opt(&c.rules, &cols, DEFAULT_MAX_ROUNDS, chunk).unwrap();
                let mut emitted: BTreeSet<(u64, u64, u64)> = BTreeSet::new();
                let mut rule_ids: BTreeSet<u32> = BTreeSet::new();
                for b in &batches {
                    rule_ids.insert(b.rule_id);
                    for i in 0..b.rows.len() {
                        emitted.insert((b.rows.s[i], b.rows.p[i], b.rows.o[i]));
                    }
                }
                let (oracle, fired) = naive_closure_full(&c.rules, &c.s, &c.p, &c.o);
                assert!(fired.iter().all(|&f| f), "seed {seed}: witness failed");
                assert_eq!(
                    rule_ids.len(),
                    c.rules.len(),
                    "seed {seed} chunk {chunk}: every rule emitted (M12-5 witness)"
                );
                // The projector emits derivations, including re-derivations
                // of base rows (dedup is A15's). So the driver's store
                // closure must equal base ∪ emitted, and every emitted row
                // must be in the oracle closure (no phantom derivations).
                let base: BTreeSet<(u64, u64, u64)> =
                    (0..c.s.len()).map(|i| (c.s[i], c.p[i], c.o[i])).collect();
                let mut union = emitted.clone();
                union.extend(base.iter().copied());
                assert!(
                    emitted.is_subset(&oracle),
                    "seed {seed} chunk {chunk}: phantom derivation"
                );
                assert_eq!(
                    union, oracle,
                    "seed {seed} chunk {chunk}: differential mismatch"
                );
            }
        }
    }

    /// Determinism court (spec 7.3 M12-6): chunk partitioning the atom-0
    /// scan — including one-row chunks — leaves every batch byte-stable:
    /// rule ids, epochs, digests, popcounts, and row tuples.
    /// Mutation that MUST flip this court: any chunk-order-dependent
    /// emission (would break A15's digest anchoring).
    #[test]
    fn determinism_court_m12_6_chunk_partition_never_changes_batch_bytes() {
        for seed in 0..5u64 {
            let c = gen_witnessed_corpus(seed);
            let cols = StatementColumns::new(&c.s, &c.p, &c.o).unwrap();
            let mut reference: Option<Vec<(u32, u64, [u8; 32], u32, Vec<(u64, u64, u64, u16)>)>> =
                None;
            for &chunk in &[1usize, 3, 64, DEFAULT_CHUNK_ROWS] {
                let batches =
                    unify_to_fixpoint_opt(&c.rules, &cols, DEFAULT_MAX_ROUNDS, chunk).unwrap();
                let mut shape = Vec::new();
                for b in &batches {
                    let mut rows = Vec::new();
                    for i in 0..b.rows.len() {
                        rows.push(b.rows.row(i));
                    }
                    shape.push((b.rule_id, b.epoch, b.digest, b.source_mask_popcount, rows));
                }
                match &reference {
                    None => reference = Some(shape),
                    Some(r) => {
                        assert_eq!(r, &shape, "seed {seed} chunk {chunk}: batch bytes diverged")
                    }
                }
            }
        }
    }

    /// Same-input/same-bindings determinism court: repeated evaluation of
    /// one (rule, store) pair yields identical `MatchSet`s — across
    /// repeated runs, fresh vs warm state-carrying scratch, and chunk
    /// sizes — the binding-map property M12-6 pins at batch level.
    /// Mutation that MUST flip this court: any state carried between runs
    /// through [`EvalScratch`] that leaks into bindings or match order.
    #[test]
    fn determinism_court_same_input_yields_identical_bindings() {
        let c = gen_witnessed_corpus(11);
        let cols = StatementColumns::new(&c.s, &c.p, &c.o).unwrap();
        let compiled: Vec<_> = c
            .rules
            .iter()
            .map(RuleCompiler::compile)
            .collect::<Result<_, _>>()
            .unwrap();
        let rule = &compiled[1]; // the join rule ?x P1 ?y . ?y P2 ?z
        let reference = unify_batch(rule, &cols, DEFAULT_CHUNK_ROWS).unwrap();
        // (a) Fresh-scratch repeat: byte-identical bindings and match order.
        let again = unify_batch(rule, &cols, DEFAULT_CHUNK_ROWS).unwrap();
        assert_eq!(reference, again, "fresh-scratch repeat diverged");
        // (b) Warm scratch previously dirtied by a DIFFERENT rule over the
        // same store: no state may leak into bindings or order.
        let mut warm = EvalScratch::default();
        let dirty = Rule {
            rule_id: 99,
            body: vec![BodyAtom::new(vw(20), cw(c.p[0]), vw(21))],
            head: [vw(21), cw(1), vw(20)],
        };
        let dc = RuleCompiler::compile(&dirty).unwrap();
        let mut dust = MatchSet::default();
        unify_batch_into(&dc, &cols, 7, &mut warm, &mut dust).unwrap();
        let mut via_warm = MatchSet::default();
        unify_batch_into(rule, &cols, DEFAULT_CHUNK_ROWS, &mut warm, &mut via_warm).unwrap();
        assert_eq!(reference, via_warm, "warm-scratch state leaked");
        // (c) Chunk size 1: same bindings, same order (M12-6 partition law
        // at match-set granularity).
        let mut chunked = MatchSet::default();
        unify_batch_into(rule, &cols, 1, &mut warm, &mut chunked).unwrap();
        assert_eq!(reference, chunked, "chunk-1 bindings diverged");
        // The named property, asserted directly: identical variable bindings.
        assert_eq!(reference.var_values, via_warm.var_values);
        assert_eq!(reference.var_values, chunked.var_values);
    }

    /// Witness-independence court (DoD 7): the witnessed generator really
    /// does produce corpora where every rule fires — the M12-5 court's
    /// non-vacuity premise, asserted over the differential court's seed
    /// range.
    #[test]
    fn witness_court_every_rule_fires_across_seeds() {
        for seed in 0..100u64 {
            let c = gen_witnessed_corpus(seed);
            let (_, fired) = naive_closure_full(&c.rules, &c.s, &c.p, &c.o);
            assert!(fired.iter().all(|&f| f), "seed {seed}");
        }
    }

    // --- Wave 4: zero-allocation hot path (spec 7.4, DoD 6) --------------

    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;
    use std::thread_local;

    thread_local! {
        static ALLOCS: Cell<usize> = const { Cell::new(0) };
        static IN_ALLOC: Cell<bool> = const { Cell::new(false) };
    }

    /// Pass-through counting allocator (house pattern:
    /// `crates/cgap-parquet/tests/page_zero_alloc.rs`). Thread-local
    /// counters keep concurrently running tests from polluting each
    /// other's measurement window. This is the `cgap-bn3` lib test
    /// binary's single `#[global_allocator]` (rustc rejects a second;
    /// the crate defines no other).
    struct Counting;

    impl Counting {
        fn bump() {
            if IN_ALLOC.with(Cell::get) {
                return;
            }
            IN_ALLOC.with(|c| c.set(true));
            ALLOCS.with(|c| c.set(c.get() + 1));
            IN_ALLOC.with(|c| c.set(false));
        }
    }

    // SAFETY: pure pass-through to System with a counted bump; the
    // contract of each method is delegated unchanged.
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            Self::bump();
            unsafe { System.alloc(layout) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            Self::bump();
            unsafe { System.realloc(ptr, layout, new_size) }
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            Self::bump();
            unsafe { System.alloc_zeroed(layout) }
        }
    }

    #[global_allocator]
    static COUNTING_ALLOC: Counting = Counting;

    /// [W2-9 unblock 2026-11-01] `pub(crate)`: noalloc.rs zero-alloc court
    /// reads this same thread-local counter (single counting substrate).
    pub(crate) fn current_allocs() -> usize {
        ALLOCS.with(Cell::get)
    }

    /// Positive control (anti-vacuity): the same measurement window around
    /// a fresh `Vec` MUST move the counter — a zero-alloc court whose
    /// counter cannot leave zero is measuring nothing. Mutation that MUST
    /// flip this court: bypassing `Counting::bump` (the counter reads 0
    /// through any allocation).
    #[test]
    fn zero_alloc_court_positive_control_counter_is_live() {
        let before = current_allocs();
        let probe: Vec<u8> = (0..64u8).collect();
        let delta = current_allocs() - before;
        println!("[za-court] positive-control allocs={delta}");
        assert!(
            delta >= 1,
            "counting allocator counted {delta} allocs for a fresh Vec — court is vacuous"
        );
        assert_eq!(probe.len(), 64);
    }

    /// Zero-allocation court (spec 7.4, DoD 6): after a warmup run sizes
    /// every buffer on this exact input, a second scan+project run over
    /// the same pre-built batch performs ZERO heap allocations, and both
    /// runs agree row-for-row with identical digests. Real kernel, real
    /// projector, real allocator — Chicago, final state asserted.
    /// Mutation that MUST flip this court: reintroducing per-call
    /// allocation in the eval path (executed shim: shadow the
    /// caller-owned scratch in `atom_local_match_into` with fresh
    /// `Vec::new()`s — the measured delta jumps to thousands and the
    /// court FAILS; see the Wave-4 lane report for the receipt).
    #[test]
    fn zero_alloc_court_scan_project_hot_path_allocates_zero_after_warmup() {
        // Pre-built 4096-row batch; the full 5-rule corpus (every rule
        // class of spec 7.1). Rule compilation happens OUTSIDE the
        // measured window (spec 7.4: compilation may allocate once).
        let rules = rule_set();
        let mut s = Vec::new();
        let mut p = Vec::new();
        let mut o = Vec::new();
        for i in 0..4096u64 {
            let subj = cw(1 + i % 7);
            s.push(subj);
            if i % 5 == 0 {
                // Self-loop feeds the self-join rule.
                p.push(cw(200));
                o.push(subj);
            } else {
                p.push(cw(100 + (i % 2) * 100));
                o.push(cw(1 + (i * 3) % 11));
            }
        }
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let compiled: Vec<CompiledRule> = rules
            .iter()
            .map(RuleCompiler::compile)
            .collect::<Result<_, _>>()
            .unwrap();

        let projector = ConsequentProjector::new();
        let mut scratch = EvalScratch::default();
        let mut out = MatchSet::default();
        // FA3 radix-default flip: lease the probe so the L3 radix tables
        // (bucket_head/next_chain/key_set/mask_s/cand caches) are warmed
        // by the passes below and the measured window stays zero-alloc.
        // A fresh-per-call probe (`unify_batch_into`) would rebuild them
        // inside the measured window by design.
        let mut probe = JoinProbeScratch::default();
        debug_assert!(probe.radix, "leased probe must default to radix");
        let mut rows = crate::project::DeltaRows::default();
        let mut canon = Vec::new();

        // Warmup: per rule, size every buffer on this exact input, and
        // record the reference emission (rows + digest). TWO passes: the
        // scan's cur/nxt double buffer carries per-role capacity, so the
        // first pass warms the match-side buffers and the second pass
        // warms the expansion-side reserve — steady state (zero allocs)
        // is the third and later identical runs.
        let mut reference: Vec<([u8; 32], Vec<(u64, u64, u64, u16)>)> = Vec::new();
        for _pass in 0..2 {
            for c in &compiled {
                unify_batch_probe(
                    c,
                    &cols,
                    DEFAULT_CHUNK_ROWS,
                    &mut scratch,
                    &mut out,
                    &mut probe,
                )
                .unwrap();
                let (d, _) = projector
                    .emit_into(c, 1, &out, &mut rows, &mut canon)
                    .unwrap();
                if _pass == 0 {
                    reference.push((d, (0..rows.len()).map(|i| rows.row(i)).collect()));
                }
            }
        }

        // Pre-reserve the capture buffers from the warmup's exact shapes:
        // every allocation the capture could need happens here, outside
        // the measured window.
        let mut measured: Vec<([u8; 32], Vec<(u64, u64, u64, u16)>)> = Vec::new();
        for (d, rows_snapshot) in &reference {
            measured.push((*d, Vec::with_capacity(rows_snapshot.len())));
        }

        // Measured window, split into phases so a leak localizes itself:
        // (a) scan only, (b) project only, (c) capture.
        let mut scan_delta = 0usize;
        let mut project_delta = 0usize;
        let mut capture_delta = 0usize;
        for (k, c) in compiled.iter().enumerate() {
            let before = current_allocs();
            unify_batch_probe(
                c,
                &cols,
                DEFAULT_CHUNK_ROWS,
                &mut scratch,
                &mut out,
                &mut probe,
            )
            .unwrap();
            scan_delta += current_allocs() - before;

            let before = current_allocs();
            let (d, _) = projector
                .emit_into(c, 1, &out, &mut rows, &mut canon)
                .unwrap();
            project_delta += current_allocs() - before;

            let before = current_allocs();
            let slot = &mut measured[k];
            slot.0 = d;
            for i in 0..rows.len() {
                slot.1.push(rows.row(i));
            }
            capture_delta += current_allocs() - before;
        }
        let delta = scan_delta + project_delta + capture_delta;
        println!(
            "[za-court] scan={scan_delta} project={project_delta} capture={capture_delta} total={delta}"
        );
        assert_eq!(
            delta, 0,
            "scan+project hot path allocated {delta} times after warmup (scan={scan_delta} project={project_delta} capture={capture_delta})"
        );
        assert_eq!(measured, reference, "warm run diverged from the warmup run");
    }

    /// Stress court (Wave 4 hardening): the fixpoint driver over a
    /// 256-row store with the full rule corpus converges within an
    /// explicit round budget, derives rows, and the emitted batch
    /// sequence is byte-identical (ids, epochs, digests, popcounts)
    /// across chunk sizes 1/64/256 — M12-6 partition independence at
    /// scale.
    /// Mutation that MUST flip this court: any chunk-order-dependent
    /// emission or a budget overrun.
    #[test]
    fn stress_court_fixpoint_converges_and_stays_chunk_stable_at_256_rows() {
        let mut s = Vec::new();
        let mut p = Vec::new();
        let mut o = Vec::new();
        for i in 0..256u64 {
            let subj = cw(1 + i % 48);
            s.push(subj);
            if i % 5 == 0 {
                p.push(cw(200));
                o.push(subj);
            } else {
                p.push(cw(100 + (i % 2) * 100));
                o.push(cw(1 + (i * 7 + 3) % 48));
            }
        }
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let rules = rule_set();
        let mut reference: Option<Vec<(u32, u64, [u8; 32], u32)>> = None;
        for &chunk in &[1usize, 64, 256] {
            let batches =
                unify_to_fixpoint_opt(&rules, &cols, 200, chunk).expect("fixpoint converges");
            let shape: Vec<(u32, u64, [u8; 32], u32)> = batches
                .iter()
                .map(|b| (b.rule_id, b.epoch, b.digest, b.source_mask_popcount))
                .collect();
            let total_rows: usize = batches.iter().map(|b| b.rows.len()).sum();
            assert!(total_rows > 0, "chunk {chunk}: no derivations");
            match &reference {
                None => reference = Some(shape),
                Some(r) => assert_eq!(r, &shape, "chunk {chunk}: batch shape diverged"),
            }
        }
    }
    // ====================================================================
    // Wave 9: cross-lane scale court (wave-9 slice item "A20 1M-row
    // stress"): the fixpoint driver + projector at one million rows —
    // convergence, closed-form derivation expectations, and M12-6
    // chunk-partition stability AT SCALE.
    // ====================================================================

    /// One-million-row stress court (wave 9): 1,000,000-row store, three
    /// single-atom rules with controlled selectivity whose expected closure
    /// is computable in closed form by an INDEPENDENT single-pass model
    /// (the court's oracle — same role as the naive unifier, no kernel
    /// machinery reused). Asserts:
    ///   1. convergence inside the round budget (Ok, never BudgetExceeded);
    ///   2. per-rule, per-round popcounts equal the model's match counts;
    ///   3. the derived distinct set equals the model's set (no phantom,
    ///      no missing derivation at 1M scale);
    ///   4. epochs are 1 then 2 per rule (round 2 re-emits, adds nothing);
    ///   5. batch shapes — ids, epochs, digests, popcounts — are identical
    ///      across chunk sizes 262144 and DEFAULT_CHUNK_ROWS (M12-6 at 1M).
    /// Mutation that MUST flip this court: silently skipping one of the
    /// three rules (the derived-set equality drops below the model), any
    /// mask/join corruption at scale, or a budget bypass.
    #[test]
    fn stress_court_1m_rows_converges_chunk_stable_matches_closed_form_model() {
        const ROWS: u64 = 1_000_000;
        const P1: u64 = 100;
        const P2: u64 = 200;
        const P4: u64 = 400;
        const BUDGET: u32 = 10;

        let s: Vec<u64> = (0..ROWS).map(|i| cw(1 + i % 7)).collect();
        let p: Vec<u64> = (0..ROWS)
            .map(|i| if i % 5 == 0 { P2 } else { P1 })
            .collect();
        let o: Vec<u64> = (0..ROWS)
            .map(|i| {
                if i % 5 == 0 {
                    cw(1 + i % 7)
                } else {
                    cw(1 + (i * 3) % 11)
                }
            })
            .collect();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();

        let rules = vec![
            // R0: ?x P1 ?y => ?x P4 ?y — renaming, no new join fuel.
            Rule {
                rule_id: 0,
                body: vec![BodyAtom::new(vw(10), cw(P1), vw(11))],
                head: [vw(10), cw(P4), vw(11)],
            },
            // R1: ?x P2 ?x => ?x P4 c(1) — self-join.
            Rule {
                rule_id: 1,
                body: vec![BodyAtom::new(vw(10), cw(P2), vw(10))],
                head: [vw(10), cw(P4), cw(1)],
            },
            // R2: (c(2), P1, ?y) => (?y, P4, c(2)) — constant-bound.
            Rule {
                rule_id: 2,
                body: vec![BodyAtom::new(cw(2), cw(P1), vw(11))],
                head: [vw(11), cw(P4), cw(2)],
            },
        ];

        // Independent model: one O(N) pass, straight row scan — no kernel
        // machinery (the court's oracle).
        let mut n_p1 = 0u64;
        let mut n_self = 0u64;
        let mut n_r2 = 0u64;
        let mut model: BTreeSet<(u64, u64, u64)> = BTreeSet::new();
        for i in 0..ROWS as usize {
            let (sv, pv, ov) = (s[i], p[i], o[i]);
            if pv == P1 {
                n_p1 += 1;
                model.insert((sv, P4, ov));
            } else {
                n_self += 1;
                model.insert((sv, P4, cw(1)));
            }
            if sv == cw(2) && pv == P1 {
                n_r2 += 1;
                model.insert((ov, P4, cw(2)));
            }
        }
        assert!(n_p1 > 500_000, "model selectivity sanity: n_p1={n_p1}");
        assert!(
            n_self == 200_000,
            "model selectivity sanity: n_self={n_self}"
        );
        assert!(n_r2 > 50_000, "model selectivity sanity: n_r2={n_r2}");

        let mut reference: Option<Vec<(u32, u64, [u8; 32], u32)>> = None;
        for chunk in [262_144usize, DEFAULT_CHUNK_ROWS] {
            let batches = unify_to_fixpoint_opt(&rules, &cols, BUDGET, chunk)
                .expect("1M fixpoint converges within budget");
            assert_eq!(batches.len(), 6, "3 rules x 2 rounds");
            for (k, b) in batches.iter().enumerate() {
                let round = k / 3 + 1;
                assert_eq!(b.epoch, round as u64, "batch {k} epoch");
                let expect_popcount = match b.rule_id {
                    0 => n_p1,
                    1 => n_self,
                    2 => n_r2, // identical in both rounds: P4 rows match no body
                    other => panic!("unexpected rule id {other}"),
                };
                assert_eq!(b.source_mask_popcount as u64, expect_popcount, "batch {k}");
                assert!(b.rows.len() > 0, "every rule derives at 1M scale");
            }
            // Distinct derived set == model set (no phantom, no missing).
            let mut derived: BTreeSet<(u64, u64, u64)> = BTreeSet::new();
            for b in &batches {
                for i in 0..b.rows.len() {
                    let (sv, pv, ov, fl) = b.rows.row(i);
                    assert_eq!(fl, 0);
                    assert_eq!(pv, P4, "all derivations carry the P4 marker");
                    derived.insert((sv, pv, ov));
                }
            }
            assert_eq!(derived, model, "derived set vs closed-form model");
            let shape: Vec<(u32, u64, [u8; 32], u32)> = batches
                .iter()
                .map(|b| (b.rule_id, b.epoch, b.digest, b.source_mask_popcount))
                .collect();
            match &reference {
                None => reference = Some(shape),
                Some(r) => assert_eq!(r, &shape, "1M: batch shape diverged at chunk {chunk}"),
            }
        }
    }
}
