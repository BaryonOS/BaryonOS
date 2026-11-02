OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { entry PT_LOAD FLAGS(5); startup PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); data PT_LOAD FLAGS(6); code PT_LOAD FLAGS(5); }
SECTIONS {
  . = 0x40000000;
  .text.entry : ALIGN(4096) { KEEP(*(.text.entry)) } :entry
  ASSERT(SIZEOF(.text.entry) > 0 && SIZEOF(.text.entry) <= 0x1000, "compositor entry exceeds one page")
  . = 0x40010000;
  .app.startup : ALIGN(4096) { KEEP(*(.app.startup)) } :startup
  ASSERT(app_startup == 0x40010000 && SIZEOF(.app.startup) == 1024, "compositor startup layout moved")
  . = 0x40020000;
  .rodata : ALIGN(4096) { *(.data.rel.ro .data.rel.ro.*) *(.rodata .rodata.*) } :readonly
  ASSERT(SIZEOF(.rodata) <= 0x80000, "window compositor readonly data exceeds512KiB")
  . = 0x40100000;
  .data : ALIGN(4096) { *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(. <= 0x40700000, "window compositor mutable data exceeds6MiB")
  . = 0x41000000;
  .text : ALIGN(4096) { *(.text .text.*) } :code
  ASSERT(SIZEOF(.text) <= 0x80000, "window compositor code exceeds512KiB")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
