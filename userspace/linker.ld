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
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(SIZEOF(.bss) == 0, "probe has unexpected BSS")
  .rodata : { *(.rodata .rodata.*) } :text
  ASSERT(SIZEOF(.rodata) == 0, "probe requires a separate NX rodata segment")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
