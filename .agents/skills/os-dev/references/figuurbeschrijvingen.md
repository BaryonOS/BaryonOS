# Text descriptions of the PDF figures

These descriptions are based on visual inspection of the supplied PDF. They are added accessibility text, not original book text.

## Page 12

Figure 2.1 — Example of the boot process. The four blocks and arrows read exactly: BIOS → GRUB1 → GRUB2 → OS. The caption states that each block is a program. This describes the source's boot example; the labels are not renamed or equated with another boot route.


## Page 34

Figure 5.1 — Translation of logical addresses to linear addresses. At the top are a 16-bit Segment selector (bits 15–0) and a 32-bit Offset (Effective address) (bits 31–0). The selector points to a Segment descriptor. The Base address from that descriptor and the offset lead to an addition symbol. The output is a 32-bit Linear address (bits 31–0). This description follows the visible figure; checks discussed in the surrounding text are not additional blocks in the drawing.


## Page 54

Figure 9.1 — Translating virtual addresses (linear addresses) to physical addresses. A 32-bit address is divided into Directory (bits 31–22; 10 bits), Table (bits 21–12; 10 bits) and Offset (bits 11–0; 12 bits). CR3 points to the Page directory using a 20-bit address field. The directory index selects a PDE (PS=0); a 20-bit address field points to the Page table. The table index selects the PTE. The 20-bit address field from the PTE and the 12-bit offset determine the Physical addr position within the illustrated 4KB Page frame. The page also discusses 4 MB pages; the drawing itself shows the path with PDE PS=0 and a separate page table.
