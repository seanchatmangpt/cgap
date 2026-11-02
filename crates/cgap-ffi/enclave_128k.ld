/* enclave_128k.ld — lane D3 (v26.11.2, charter Deliverable 4.3) 128 KiB
 * enclave budget linker script. Fail-closed: anything that does not fit
 * inside the 128 KiB ENCLAVE_FLASH region refuses AT LINK TIME — an
 * oversize enclave image can never be minted, only refused.
 *
 * Court (docs/jira/v26.11.2/waves/w5-linker-budgets.md): baseline link
 * green; mutant (synthetic oversize .text / region overflow) link MUST
 * fail; restores sha256-exact.
 */
MEMORY
{
  ENCLAVE_FLASH (rx) : ORIGIN = 0x10000000, LENGTH = 128K
  ENCLAVE_SRAM  (rw) : ORIGIN = 0x30000000, LENGTH = 64K
}
ENTRY(cgap_xip_start)
SECTIONS
{
  .text : { KEEP(*(.text.cgap_*)) *(.text*) } > ENCLAVE_FLASH
  .rodata : { *(.rodata*) *(.srodata*) *(.oabi) *(.bn3) } > ENCLAVE_FLASH

  .data : { *(.data*) *(.sdata*) } > ENCLAVE_SRAM AT>ENCLAVE_FLASH
  .bss  : { *(.bss*) *(.sbss*) *(COMMON) } > ENCLAVE_SRAM
  _stack_top = ORIGIN(ENCLAVE_SRAM) + LENGTH(ENCLAVE_SRAM);
}
/* Charter Deliverable 4.3 — the named budget gate (fail-closed). */
ASSERT(SIZEOF(.text) <= 131072, "cgap ENCLAVE 128K: .text exceeds the 131072-byte enclave flash budget")
/* Region-exhaustion backstop: flash is text+rodata combined; the region
 * arithmetic itself refuses an oversize image even if .text alone fits. */
ASSERT((SIZEOF(.text) + SIZEOF(.rodata)) <= 131072, "cgap ENCLAVE 128K: text+rodata exceeds the 131072-byte enclave flash budget")
ASSERT(SIZEOF(.data) + SIZEOF(.bss) <= 65536, "cgap ENCLAVE 128K: data+bss exceeds the 65536-byte enclave SRAM budget")
/* Lane AE9 (v26.11.2 countermeasure lane-09): linker SYMBOL PINS so the
 * deployment gate can cross-check the linked image against the budget
 * script (scripts/enclave_budget_gate.py --verify-symbols). Direct
 * assignments (not PROVIDE) — a pin that only materializes when referenced
 * could be absent from the ELF, and the gate must be able to FAIL on a
 * missing pin, never pass on one.
 *
 * Anti-vacuity: .text <= 96 KiB and .bss <= 16 KiB are per-section
 * sub-budgets the whole-envelope ASSERT above CANNOT express (a 100 KiB
 * .text with 20 KiB rodata passes text+rodata<=128K only via luck of the
 * split); a mutant whose .bss exceeds 16384 B must refuse AT LINK TIME on
 * the named message below — witnessed in
 * docs/jira/v26.11.2/countermeasures/lane-09-linker-asserts.md.
 */
_stext = ADDR(.text);
_etext = _stext + SIZEOF(.text);
_sbss = ADDR(.bss);
_ebss = _sbss + SIZEOF(.bss);
ASSERT(SIZEOF(.text) <= 98304, "cgap ENCLAVE 128K: .text exceeds the 98304-byte (96 KiB) per-section sub-budget")
ASSERT(SIZEOF(.bss) <= 16384, "cgap ENCLAVE 128K: .bss exceeds the 16384-byte (16 KiB) per-section sub-budget")
ASSERT((SIZEOF(.text) + SIZEOF(.rodata) + SIZEOF(.data) + SIZEOF(.bss)) <= 131072, "cgap ENCLAVE 128K: total image exceeds the 131072-byte enclave budget")
