OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { entry PT_LOAD FLAGS(5); startup PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); data PT_LOAD FLAGS(6); code PT_LOAD FLAGS(5); }
SECTIONS {
  . = 0x40000000;
  .text.entry : ALIGN(4096) { KEEP(*(.text.entry)) } :entry
  ASSERT(SIZEOF(.text.entry) > 0 && SIZEOF(.text.entry) <= 0x1000, "entry exceeds one page")
  . = 0x40010000;
  .startup : ALIGN(4096) { KEEP(*(.app.startup)) } :startup
  ASSERT(app_startup == 0x40010000 && SIZEOF(.startup) == 1024, "startup ABI moved")
  . = 0x40020000;
  /* Consume static font pointer tables before the later writable wildcard. */
  .rodata : ALIGN(4096) { *(.data.rel.ro .data.rel.ro.*) *(.rodata .rodata.*) } :readonly
  /* Shared proportional UI atlas: bounded read-only pages, never writable. */
  ASSERT(SIZEOF(.rodata) <= 0x80000, "readonly exceeds 512 KiB")
  . = 0x40100000;
  .data : ALIGN(4096) { *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : ALIGN(4096) { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(. <= 0x40140000, "data/BSS exceeds 256 KiB")
  . = 0x40140000;
  .text : ALIGN(4096) { *(.text .text.*) } :code
  ASSERT(SIZEOF(.text) <= 0x20000, "code exceeds 128 KiB")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
