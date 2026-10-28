OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); }
SECTIONS {
  . = 0x40000000;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  ASSERT(SIZEOF(.text) > 0 && SIZEOF(.text) <= 0x10000, "qsim text overlaps report")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.qsim.header)) *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(qsim_startup == 0x40010000, "qsim report ABI moved")
  ASSERT(. <= 0x40011000, "qsim report and allocator cursor exceed one page")
  . = 0x40011000;
  .rodata : ALIGN(4096) { *(.rodata .rodata.*) } :readonly
  ASSERT(SIZEOF(.rodata) <= 0x8000, "qsim rodata exceeds budget")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
