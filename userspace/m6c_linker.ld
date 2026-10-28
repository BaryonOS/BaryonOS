OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); }
SECTIONS {
  . = 0x40000000;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  ASSERT(SIZEOF(.text) > 0 && SIZEOF(.text) <= 0x10000, "M6c text overlaps data")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.m6.header)) *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(. <= 0x40011000, "M6c startup/GOT exceeds control page")
  . = 0x40011000;
  .buffers : { KEEP(*(.m6.buffers)) } :data
  ASSERT(m6_startup == 0x40010000, "M6c startup ABI moved")
  ASSERT(m6_message == 0x40011000 && m6_output == 0x40012000, "M6c buffers moved")
  ASSERT(SIZEOF(.buffers) == 12288, "M6c buffers require three pages")
  ASSERT(m6_sector == 0x40013000, "M6c sector buffer moved")
  . = 0x40014000;
  .rodata : ALIGN(4096) { KEEP(*(.rodata.m6)) *(.rodata .rodata.*) } :readonly
  ASSERT(SIZEOF(.rodata) > 0 && SIZEOF(.rodata) <= 4096, "M6c rodata exceeds one page")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
