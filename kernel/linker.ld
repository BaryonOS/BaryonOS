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
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  . = ALIGN(4096);
  .rodata : { *(.rodata .rodata.*) } :rodata
  . = ALIGN(4096);
  /* LLVM emits a small GOT with link-time resolved pointers between Rust crates.
     Include it explicitly; the build inspector rejects all runtime relocations. */
  .data : { *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : ALIGN(16) { *(.bss .bss.*) *(COMMON) } :data
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) }
  /* Boundaries derive from section addresses, not the location counter: cargo
     merges an enclosing checkout's .cargo/config.toml, passing this script twice,
     and lld then resolved `sym = .` assignments to the base address. */
  __kernel_start = ADDR(.text);
  __text_end = ADDR(.rodata);
  __rodata_end = ADDR(.data);
  __kernel_end = ALIGN(ADDR(.bss) + SIZEOF(.bss), 4096);
  ASSERT(__kernel_start < __text_end && __text_end <= __rodata_end && __rodata_end < __kernel_end,
         "kernel layout symbols out of order")
  ASSERT(__kernel_end <= 0x03000000, "kernel exceeds reserved span")
}
