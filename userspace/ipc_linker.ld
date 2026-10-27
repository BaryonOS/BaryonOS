OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS {
  text PT_LOAD FLAGS(5);
  data PT_LOAD FLAGS(6);
  readonly PT_LOAD FLAGS(4);
}
SECTIONS {
  . = 0x40000000;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  ASSERT(SIZEOF(.text) > 0 && SIZEOF(.text) <= 0x10000,
         "IPC text overlaps fixed control address")
  . = 0x40010000;
  .data : ALIGN(4096) { KEEP(*(.data.ipc)) *(.data .data.*) } :data
  ASSERT(ipc_control == 0x40010000 && SIZEOF(.data) == 0x3000,
         "IPC control/buffer ABI changed")
  . = 0x40013000;
  .rodata : ALIGN(4096) { KEEP(*(.rodata.ipc)) *(.rodata .rodata.*) } :readonly
  ASSERT(ipc_readonly == 0x40013000 && SIZEOF(.rodata) == 4096,
         "IPC read-only boundary ABI changed")
  .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
  ASSERT(SIZEOF(.bss) == 0, "IPC fixture has unexpected BSS")
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
