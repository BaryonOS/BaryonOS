OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); code PT_LOAD FLAGS(5); }
SECTIONS {
  . = 0x40000000;
  .text.entry : ALIGN(4096) { KEEP(*(.text.entry)) } :text
  ASSERT(SIZEOF(.text.entry) > 0 && SIZEOF(.text.entry) <= 0x1000, "input entry exceeds one page")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.app.startup)) *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(app_startup == 0x40010000, "input startup layout moved")
  ASSERT(. <= 0x40018000, "input app data exceeds one page")
  . = 0x40020000;
  .rodata : ALIGN(4096) { *(.rodata .rodata.*) } :readonly
  ASSERT(SIZEOF(.rodata) <= 0x18000, "input app rodata exceeds48KiB")
  . = 0x40040000;
  .text : ALIGN(4096) { *(.text .text.*) } :code
  ASSERT(SIZEOF(.text) <= 0x40000, "input app code exceeds128KiB")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
