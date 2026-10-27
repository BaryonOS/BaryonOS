OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS {
  text   PT_LOAD FLAGS(5);
  rodata PT_LOAD FLAGS(4);
  data   PT_LOAD FLAGS(6);
}
SECTIONS {
  . = 0x02000000;
  __kernel_start = .;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  . = ALIGN(4096);
  __text_end = .;
  .rodata : { *(.rodata .rodata.*) } :rodata
  . = ALIGN(4096);
  __rodata_end = .;
  /* LLVM emits a small GOT with link-time resolved pointers between Rust crates.
     Include it explicitly; the build inspector rejects all runtime relocations. */
  .data : { *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : ALIGN(16) { *(.bss .bss.*) *(COMMON) } :data
  __kernel_end = ALIGN(4096);
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) }
  ASSERT(__kernel_end <= 0x03000000, "kernel exceeds reserved span")
}
