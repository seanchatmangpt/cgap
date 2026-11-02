/* xip.ld — lane A19 (v26.11.1) libcgap XIP flash-residency linker script.
 *
 * Spec: docs/jira/v26.11.1/lanes/A19-libcgap-ffi-xip.md section 5 (the
 * PROPOSED sketch, reviewed against the ESP32-S3 memory map). Flash-resident
 * text + rodata; SRAM-resident data/bss/stack. Map is ESP32-S3-shaped; the
 * RISC-V variant (ESP32-C3 rv32imc) swaps the flash origin/len per SoC
 * datasheet.
 *
 * Build-level court (sovereign.rs::xip_courts): a link with no .oabi/.bn3
 * input sections MUST fail right here on the ASSERTs — a script whose
 * ASSERTs cannot fire is decoration, so the court runs the section-absent
 * mutant every session and refuses to pass if the link still succeeds.
 */
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x42000020, LENGTH = 4M   /* app partition mmap */
  SRAM  (rwx) : ORIGIN = 0x3FC80000, LENGTH = 320K
}
ENTRY(cgap_xip_start)
SECTIONS
{
  .text : { KEEP(*(.text.cgap_*)) *(.text*) } > FLASH

  /* Sovereign rule tables + networks live in flash, page-aligned (the
   * ESP32-S3 mmap page is 64 KiB; misalignment is a link-time error via the
   * region overflow, never a runtime surprise). */
  .oabi : ALIGN(64K) { KEEP(*(.oabi)) } > FLASH
  .bn3  : ALIGN(64K) { KEEP(*(.bn3))  } > FLASH

  .rodata : { *(.rodata*) *(.srodata*) } > FLASH

  /* Latent tier anchors (lane D3): empty until .t2/.t3 input sections
   * land; SIZEOF(.t2)/SIZEOF(.t3) in the budget ASSERTs below are then
   * always defined. */
  .t2 : { *(.t2) } > FLASH
  .t3 : { *(.t3) } > FLASH

  .data : { *(.data*) *(.sdata*) } > SRAM AT>FLASH
  .bss  : { *(.bss*) *(.sbss*) *(COMMON) } > SRAM
  _stack_top = ORIGIN(SRAM) + LENGTH(SRAM);
}
ASSERT(SIZEOF(.oabi) > 0, "cgap XIP: .oabi section absent")
ASSERT(SIZEOF(.bn3) > 0, "cgap XIP: .bn3 section absent")

/* ---- Per-tier budget ASSERTs (lane D3, v26.11.2 charter Deliverable 4.3).
 * Tier mapping (recorded divergence: the D3 dispatch names four budgets but
 * not their section anchors; this mapping is the D3 definition):
 *   gate tier  <= 16K  -> .oabi   (gate rule tables; section EXISTS, active)
 *   tier1      <= 48K  -> .bn3    (BN3 network; section EXISTS, active)
 *   tier2      <= 48K  -> .t2     (section NOT YET PLACED; latent OR-zero
 *                          form so the ASSERT evaluates every link and
 *                          activates the moment the section lands —
 *                          never a silent skip)
 *   tier3      <= 12K  -> .t3     (same latent form)
 * Anti-vacuity: the gate/tier1 ASSERTs are flipped every session by the
 * .oabi/.bn3 section-absent mutant (presence leg) and by the oversize
 * .oabi mutant court in docs/jira/v26.11.2/waves/w5-linker-budgets.md
 * (budget leg). ---- */
ASSERT(SIZEOF(.oabi) == 0 || SIZEOF(.oabi) <= 16384, "cgap XIP: gate-tier .oabi budget 16384 exceeded")
ASSERT(SIZEOF(.bn3) == 0 || SIZEOF(.bn3) <= 49152, "cgap XIP: tier1 .bn3 budget 49152 exceeded")
ASSERT(SIZEOF(.t2) == 0 || SIZEOF(.t2) <= 49152, "cgap XIP: tier2 .t2 budget 49152 exceeded (latent: .t2 not yet placed)")
ASSERT(SIZEOF(.t3) == 0 || SIZEOF(.t3) <= 12288, "cgap XIP: tier3 .t3 budget 12288 exceeded (latent: .t3 not yet placed)")
