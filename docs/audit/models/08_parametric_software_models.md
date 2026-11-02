# AU8 — Parametric Software Models (Phase 2, 2026-11-02)

## SECTION 1 — MEASURED INPUTS (AU1-cited)
Source: docs/audit/metrics/01_cgap_topology_and_sloc.md (AU1, measured at HEAD 2026-11-02).
- Kernel Rust src SLOC (15 kernel crates, AU1 §3 total): **65,776 LOC → 65.8 KSLOC** (range 63–68 given AU1 cfg(test)-marker approximation).
- Rust tooling/facade + cli + benches: ~35,200 LOC (not modeled; tooling discipline class lower).
- Python tooling scripts/*.py: **12,321 LOC → 12.3 KSLOC**.
- Constraints (AU1 §4–5): zero external deps, `#![no_std]` in 10 kernel crates → bare-metal-class; root Cargo.toml, 24 members, 22 leaves.
- Platform context: bare-metal RISC-V, 128 KiB storage (stated lane constraint).

## SECTION 2 — MODEL ASSUMPTIONS (stated; every parameter named, sources cited)
### 2a. COCOMO II.2000 embedded/projected mode (source: Boehm et al., "Software Cost Estimation with COCOMO II", 2000)
| Param | Rating | Value used | Basis |
|---|---|---|---|
| SIZE | measured | 65.8 KSLOC (63–68) | AU1 §3 |
| CPLX | Extra-High | 1.66 | COCOMO II.2000 table; ISR/constant-time class |
| TIME | Extra-High | 1.43 | table; hard real-time |
| STOR | Extra-High | 1.46 | table; 128 KiB budget |
| PREX | Very-High | ~0.87 | table; team familiar with platform |
| PEX | Very-High | ~0.91 | table; 95th-percentile capability |
| PERS | Very-High | ~0.81 | table; elite personnel |
| A=2.94, B=0.91 | calibrated | 2.94 / 0.91 | COCOMO II.2000 constants |
| ΣSF (RESL,TEAM,PMAT,PLEX,PREX sf) | constrained | +10 to +18 → E = B+0.01ΣSF = **1.01–1.09** | stated; embedded post-architecture profile |
| ΠEM | derived | 1.66×1.43×1.46×0.87×0.91×0.81 ≈ **2.2** (range 1.8–2.6) | product of above; rating uncertainty ±1 level |
Formula: PM = A × KSLOC^E × ΠEM. Ranges shown, no point-precision claimed.
### 2b. Putnam SLIM-style power curve (source: Putnam, "Software Life Cycle Models", 1978; Boehm 1981)
- MY = PI × KSLOC^1.2, exponent 1.2 stated (Putnam-style productivity curve).
- PI range: **0.8–2.0 person-months/KSLOC^1.2** for safety-critical bare-metal (stated assumption, not measured).
### 2c. Verification uplift (AU7 rigor class, DO-178C-like)
- NOT a baseline COCOMO parameter (COCOMO assumes conventional V&V). Industry evidence: verification is 50–100% of total effort at this rigor (stated, e.g., DO-178C program experience; several suppliers report test/V&V ≈ 50–70% of cost).
- Applied uplift multiplier on baseline effort: **×1.5–2.0** (stated).

## SECTION 3 — MODEL OUTPUTS (ranges, person-months / calendar)
### COCOMO II (kernel Rust only, 65.8 KSLOC)
- PM = 2.94 × 65.8^E × ΠEM; E=1.01..1.09, ΠEM=1.8..2.6
- → **400–740 person-months** (center estimate ≈ 525 PM at E≈1.05, ΠEM≈2.2).
- TDEV = 3.67 × PM^0.28 months (SCED nominal): PM 400→734 gives **20–23 calendar months** (center ≈ 21).
### Putnam power curve (kernel Rust, 65.8 KSLOC)
- MY = PI × 65.8^1.2 = PI × 152 → PI 0.8–2.0 → **120–300 person-years-equivalent person-months** i.e. **~120–300 PM**; wide by design (PI is an assumption, not a measurement).
### Verification uplift (applied to COCOMO II baseline)
- 400–740 PM × 1.5–2.0 → **600–1,480 person-months** total effort at DO-178C-like rigor.
### Python tooling (12.3 KSLOC, organic class, CPLX nominal, PERS VH)
- PM = 2.94 × 12.3^1.04 × ΠEM(~0.9–1.1) → **≈ 35–55 person-months** (order-of-magnitude only).
## Interpretation
- All outputs are RANGES driven by stated assumption tables; the single most sensitive parameter is ΣSF (E exponent) — a ±4 swing in ΣSF moves PM ~±120.
- COCOMO II and Putnam disagree by design (2–4× on baseline): Putnam's curve omits the Extra-High CPLX/TIME/STOR penalties explicitly carried by COCOMO EMs; treat COCOMO II range as primary, Putnam as sanity band.
- Baseline 400–740 PM is pre-uplift; with the AU7 rigor-class verification multiplier, planning envelope is **~600–1,480 PM** over **~20–23+ calendar months** (duration grows modestly since schedule is sublinear in effort: PM^0.28).
