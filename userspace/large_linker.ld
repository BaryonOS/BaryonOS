OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS {
  text PT_LOAD FLAGS(5);
  data PT_LOAD FLAGS(6);
}
SECTIONS {
  . = 0x40000000;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  ASSERT(SIZEOF(.text) > 0 && SIZEOF(.text) <= 0x10000,
         "probe text overlaps fixed data address")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.data.probe)) *(.data .data.*) } :data
  ASSERT(probe_data == 0x40010000, "probe data ABI moved")
  ASSERT(SIZEOF(.data) == 32, "probe data ABI must contain exactly four u64s")
  .bss (NOLOAD) : ALIGN(4096) {
    __large_bss_start = .;
    . += 33 * 4096;
    __large_bss_end = .;
  } :data
  ASSERT(__large_bss_start == 0x40011000, "large BSS start moved")
  ASSERT(SIZEOF(.bss) == 33 * 4096, "large BSS must contain 33 extra pages")
  .rodata : { *(.rodata .rodata.*) } :text
  ASSERT(SIZEOF(.rodata) == 0, "probe requires a separate NX rodata segment")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
