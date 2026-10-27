OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); }
SECTIONS {
  . = 0x40000000;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  ASSERT(SIZEOF(.text) > 0 && SIZEOF(.text) <= 0x10000, "M5 text overlaps data")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.m5.header)) *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(. <= 0x40011000, "M5 startup/GOT exceeds control page")
  . = 0x40011000;
  .buffers : { KEEP(*(.m5.buffers)) } :data
  ASSERT(m5_startup == 0x40010000, "M5 startup ABI moved")
  ASSERT(m5_message == 0x40011000 && m5_output == 0x40012000, "M5 buffers moved")
  ASSERT(SIZEOF(.buffers) == 8192, "M5 buffers require two pages")
  . = 0x40013000;
  .rodata : ALIGN(4096) { KEEP(*(.rodata.m5)) *(.rodata .rodata.*) } :readonly
  ASSERT(SIZEOF(.rodata) > 0 && SIZEOF(.rodata) <= 4096, "M5 rodata exceeds one page")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
