OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); code PT_LOAD FLAGS(5); }
SECTIONS {
  . = 0x40000000;
  .text.entry : ALIGN(4096) { KEEP(*(.text.entry)) } :text
  ASSERT(SIZEOF(.text.entry) > 0 && SIZEOF(.text.entry) <= 0x1000, "entry exceeds one page")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.ai.header)) *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(ai_startup == 0x40010000, "report header moved")
  ASSERT(. <= 0x40011000, "data exceeds one page")
  . = 0x40014000;
  .rodata : ALIGN(4096) { *(.rodata .rodata.*) } :readonly
  ASSERT(SIZEOF(.rodata) <= 0xC000, "rodata exceeds budget")
  . = 0x40020000;
  .text : ALIGN(4096) { *(.text .text.*) } :code
  ASSERT(SIZEOF(.text) <= 0x40000, "code exceeds 256 KiB")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
