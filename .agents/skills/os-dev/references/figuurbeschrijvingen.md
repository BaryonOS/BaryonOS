# Tekstbeschrijvingen van de PDF-figuren

Deze beschrijvingen zijn gebaseerd op visuele inspectie van de aangeleverde PDF. Het zijn toegevoegde toegankelijkheidsteksten, geen oorspronkelijke boektekst.

## Pagina 12

Figuur 2.1 — Example of the boot process. De vier blokken en pijlen luiden letterlijk: BIOS → GRUB1 → GRUB2 → OS. Het bijschrift zegt dat ieder blok een programma is. Dit beschrijft het bootvoorbeeld van de bron; de labels worden niet hernoemd of gelijkgesteld aan een andere bootroute.

## Pagina 34

Figuur 5.1 — Translation of logical addresses to linear addresses. Bovenaan staan een 16-bit Segment selector (bits 15–0) en een 32-bit Offset (Effective address) (bits 31–0). De selector wijst naar een Segment descriptor. Het Base address uit die descriptor en de offset gaan naar een optelsymbool. De uitvoer is een 32-bit Linear address (bits 31–0). Deze beschrijving volgt de zichtbare figuur; de controles uit de lopende tekst zijn geen extra getekende blokken.

## Pagina 54

Figuur 9.1 — Translating virtual addresses (linear addresses) to physical addresses. Een 32-bit adres is opgedeeld in Directory (bits 31–22; 10 bits), Table (bits 21–12; 10 bits) en Offset (bits 11–0; 12 bits). CR3 wijst met een 20-bit adresdeel naar de Page directory. De directoryindex selecteert een PDE (PS=0); een 20-bit adresdeel wijst naar de Page table. De table-index selecteert de PTE. Het 20-bit adresdeel uit de PTE en de 12-bit offset bepalen de positie Physical addr binnen het afgebeelde 4KB Page frame. De pagina bespreekt daarnaast 4 MB-pagina’s; de tekening zelf toont het pad met PDE PS=0 en een aparte paginatabel.
