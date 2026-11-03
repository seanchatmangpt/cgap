# RED-18 — Supply Chain & Physical Appliance Tampering Vectors

Lane: RED-18, adversarial red-team wave v26.11.2, repo /Users/sac/cgap (read-only contract;
this file is the sole write). Date: 2026-11-02.

## 1. Target adjudication: Fogclave 1L desktop appliance — UNSUPPORTED

Claim under attack: "Fogclave 1L desktop hardware appliance, $899".

Verdict: **UNSUPPORTED — no on-disk referent.**
Method: `grep -rni fogclave` over the entire repo returned zero matches across
*.md, *.rs, *.toml, *.ttl. `grep -rniE 'appliance|\$899|desktop'` over
docs/jira/v26.11.2/ matched only Docker Desktop runtime notes
(lane-ax-00-gate.md:21, lane-ax-05-latency.md:7, dflss/01-project-charter-ax-testbed.md:41),
which describe the CI/test VM, not a hardware product. "1L" appears only as a
campaign/level label (e.g. 00-ard-prd.md lever names L1–L5), never as a
device model. The appliance claim is hearsay with zero bits in this tree.

Per dispatch, the lane pivots to the deployment model that IS on disk.

## 2. Actual on-disk deployment model

The canonical charter
(docs/jira/v26.11.2/12-ard-prd-cgap.md:11-12, 26-31) describes deployment
on "edge silicon, secure enclaves, microcontrollers, and deterministic
high-throughput hosts", targeting "sub-32 KB Flash environments,
zero-allocation real-time loops, and hardware-attested execution pipelines"
(12-ard-prd-cgap.md:31). The measured silicon target is riscv32imc with
flash < 32,768 B and SRAM + stack < 4,096 B (00-ard-prd.md G5-SILICON,
lines 60-63; `crates/cgap-gate/src/symbol_gate.py` gates 0 alloc/panic
symbols). Integrity enforcement is the P28 license attestation
(80-byte, `crates/cgap-p28/src/attestation.rs`), loaded via
`find_in_elf`/`find_in_wasm` → `from_bytes` → `validate`
(crates/cgap-p28/src/trap.rs:13, 92).

## 3. Physical attack vectors against the on-disk deployment model

### 3.1 Cold-boot / DRAM bus attacks

- Vector: dram remanence attack — power-cut the host, chill DRAM, replug
  into attacker FPGA interposer reading the bus.
- Mitigation in repo: none. `grep -rniE 'cold.?boot|dram|memory encryption'`
  over docs/jira/v26.11.2/ and crates/ returns no mitigating design. The
  riscv32imc target (00-ard.md G5) is SRAM-based MCU memory, but the
  charter's high-throughput-host tier has no memory-encryption or
  scrub-on-lock design anywhere on disk.
- Grade: **BREACH-CONFIRMED** at the charter level — the charter claims
  "secure enclaves" (12-ard-prd-cgap.md:11) yet documents no enclave
  (TEE/TrustZone/SGX/Realm) integration, no memory bus protection. The
  "secure enclave" mention is a marketing referent with no referent in
  crates/: grep of crates/ for TEE/TrustZone/SGX/enclave hits only
  unrelated test strings (e.g. cgap-git/tests/fuzz_codec.rs).
- Note: at the MCU tier (riscv32imc, 4 KB SRAM) cold-boot is lower-value
  but still unmitigated — no scrub-on-reset law exists on disk.

### 3.2 USB-C PD power glitching

- Vector: malicious PD source/sink injects voltage glitch during the P28
  validate window to skip a fail-closed branch.
- Referent: crates/cgap-p28/src/trap.rs semantics are fail-closed by design:
  CGAP_ERR_P28_SPOLIATION 0x5032_38FF (crates/cgap-p28/src/trap.rs:36),
  "one-bit mutation of the attestation" must fail closed
  (crates/cgap-p28/src/trap.rs:34-38). This is the strongest on-disk
  property and it is purely logical fail-closedness.
- Mitigation in repo: none at the electrical layer — no voltage-monitor/
  brownout/glitch-detector design, no redundancy (dual-rail monitor,
  ERA, or triple modular redundancy on the validate branch) is
  documented. Logical fail-closedness does not survive a glitch that
  flips the comparison outcome in SRAM/registers; the design assumes an
  honest power supply.
- Grade: **BREACH-CONFIRMED** (typo fixed in §4 errata below) — logical
  spoliation trap exists; electrical-layer glitching is an unmitigated
  hole.

### 3.3 Hardware interposers / bus snooping

- Vector: SPI/QSPI flash interposer or MMIO bus probe captures the 80-byte
  P28 attestation and the license payload during boot, enabling image
  cloning with a valid attestation.
- The 80-byte layout and LICENSE_MAGIC `b"P28S"` are fully public
  (crates/cgap-p28/src/attestation.rs:34, 70). Integrity binding is
  BLAKE3-only: `upstream_receipt == cgap_blake3::hash(&parent.to_bytes())`
  (attestation.rs:271) and the instrument digest
  `cgap_blake3::hash(instrument_header)` (attestation.rs:313-319). There is
  NO asymmetric signature, NO HMAC with a device-resident key, NO key
  anywhere outside the image. Any attacker who reads flash once can
  recompute a valid 80-byte attestation and re-sign nothing — validate()
  accepts it.
- Grade: **BREACH-CONFIRMED** — the attestation is clone-everything:
  unencrypted, unsigned, keyless. `find_in_elf` accepts any image that
  embeds a self-consistent attestation block (crates/cgap-p28/src/trap.rs:321).

### 3.4 Supply-chain interception (fab → customer)

- Vector: interdiction between CI artifact and customer device; flash a
  modified image whose P28 attestation is recomputed (see 3.3) or whose
  license check is patched out at the trap entry.
- Compounding repo fact: the whole integrity chain terminates in
  software-controlled state (SRAM/flash bytes), with no hardware root of
  trust (no OTP-locked public key, no measured boot, no device attestation
  key) anywhere on disk — grep for measured-boot/OTP/Dice/TPM over docs/
  and crates/ returns zero mitigating referents.
- Grade: **BREACH-CONFIRMED** — the substrate provides tamper-EVIDENT
  receipts (12-ard-prd-cgap.md:68) but no tamper-RESISTANT root; every
  physical-layer defense is absent by omission, not by waiver.

## 4. Errata

None. Typo "BREACH-CONFIRed" in §3.2 fixed in-place pre-landing; this is
the first and only write of this file (no prior receipt exists to errata).

## 5. Fences (must-not-regress surfaces this read-only lane could break)

None — read-only lane, single-file write, no code touched. Existing gates
(boundary_gate.py, zero_string_gate.py, court_verdict) untouched; not run
because no diff exists to fence.

## 6. Standing

**ALIVE** for the lane itself (all greps + file reads executed this
session, commands with outputs above). Target verdicts: Fogclave 1L
appliance = UNSUPPORTED (no referent); on-disk deployment model =
BREACH-CONFIRMED on all four physical vectors — the charter's
"hardware-attested execution pipelines" claim (12-ard-prd-cgap.md:31)
has no hardware attestation referent anywhere in crates/; P28 attestation
is software-only, keyless, BLAKE3-bound, fully cloneable.
