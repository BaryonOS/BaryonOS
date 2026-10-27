# Kernelregels
Lees root-AGENTS.md, ARCHITECTURE.md en SECURITY-AND-QUANTUM.md onder docs/.
Rust no_std; elk unsafe-blok heeft lokale SAFETY-onderbouwing voor geldigheid, aliasing, alignment en levensduur. Een commentaar alleen bewijst niets: laat het tegenvoorbeelden en review doorstaan.
Beschrijf eigenaarschap van frames/pagetables, exception- en syscall-ABI, interruptstatus en lockvolgorde. Valideer userpointers en lengtes. Nooit verwijzingen naar onbeheerde guest/userspace-geheugens vasthouden over herplanning zonder geldig model.
Capabilities zijn procesgebonden kernelgevalideerde handles met rechten en geldigheid. Geen integers/pointers als onbeperkte autoriteit. Test rechtenvergroting, stale handles, uitputting en intrekking.
Houd kernel klein. Geen GUI, netwerkparser, cryptoprotocol of opslagpolicy in privileged code zonder expliciete ADR. Geen SMP uitbreiden vóór single-core-invarianten aantoonbaar zijn.
