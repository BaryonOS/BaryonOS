OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS { entry PT_LOAD FLAGS(5); startup PT_LOAD FLAGS(6); readonly PT_LOAD FLAGS(4); data PT_LOAD FLAGS(6); code PT_LOAD FLAGS(5); }
SECTIONS {
 . = 0x40000000;
 .text.entry : ALIGN(4096) { KEEP(*(.text.entry)) } :entry
 ASSERT(SIZEOF(.text.entry)>0 && SIZEOF(.text.entry)<=0x1000,"desktop init entry exceeds one page")
 . = 0x40010000;
 .app.startup : ALIGN(4096) { KEEP(*(.app.startup)) } :startup
 ASSERT(app_startup==0x40010000 && SIZEOF(.app.startup)==1024,"desktop init startup moved")
 . = 0x40020000;
 .rodata : ALIGN(4096) { *(.data.rel.ro .data.rel.ro.*) *(.rodata .rodata.*) } :readonly
 ASSERT(SIZEOF(.rodata)<=0x10000,"desktop init rodata exceeds64KiB")
 . = 0x40040000;
 .data : ALIGN(4096) { *(.data .data.*) *(.got .got.*) } :data
 .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } :data
 ASSERT(.<=0x40050000,"desktop init writable data exceeds64KiB")
 . = 0x40060000;
 .text : ALIGN(4096) { *(.text .text.*) } :code
 ASSERT(SIZEOF(.text)<=0x20000,"desktop init text exceeds128KiB")
 /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) *(.llvm_addrsig) }
}
