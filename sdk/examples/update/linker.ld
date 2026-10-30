OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); code PT_LOAD FLAGS(5); }
SECTIONS {
  . = 0x40000000;
  .text.entry : ALIGN(4096) { KEEP(*(.text.entry)) } :text
  ASSERT(SIZEOF(.text.entry) > 0 && SIZEOF(.text.entry) <= 0x1000, "entry stub exceeds one page")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.app.startup)) *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(app_startup == 0x40010000, "existing startup layout moved")
  ASSERT(. <= 0x40011000, "app data exceeds one page")
  . = 0x40014000;
  .rodata : ALIGN(4096) { *(.rodata .rodata.*) } :readonly
  ASSERT(SIZEOF(.rodata) <= 0x1C000, "service rodata exceeds the 112 KiB budget")
  . = 0x40030000;
  .text : ALIGN(4096) { *(.text .text.*) } :code
  ASSERT(SIZEOF(.text) <= 0x40000, "service code exceeds the 256 KiB budget")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
