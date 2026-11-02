# Exact begrensde x86-64-geheugenintrinsics

2 november 2026; basis `ecb9964de727561a900ed93bc86ad3d5847dfe6b`.
Alleen `kernel/src/memory.rs`, gerichte hostharnas en deze notitie. Geen
snapshot-, capability-, syscall-, cache- of gebruikersinterfacewijziging.

De echte venstersessie gaf een ernstig vertragingssignaal met de nieuwe
exacte snapshots. Overlappende VM-belasting verhindert een precieze ratio.
Een mogelijke oorzaak is herhaalde bytegewijze emulatie in TCG; dit is nog geen
aangetoonde attributie van alle vertraging.

## Contract

`memcpy` vereist de bestaande C-intrinsicvoorwaarden: `n` leesbare en schrijfbare
bytes in geldige, niet-overlappende normale RAM-regio's. De routine kopieert
precies `n/8` volledige woorden met `rep movsq`, gevolgd door precies `n%8`
bytes met `rep movsb`. x86-64 staat deze onuitgelijnde woordtoegang toe. RSI,
RDI en RCX zijn expliciete inout-registers; het tailregister kan niet met deze
early-clobbers overlappen. Er is geen overread, overwrite of vooruitgelezen
padding. De oorspronkelijke bestemmingspointer wordt teruggegeven. `cld`
loopt ook bij lengte nul.

`memcmp` leest alleen een volledig woord als er minstens acht geldige bytes
resteren. Inline MOV-instructies voorkomen een optimizergegenereerde recursieve
memcmp-call. Bij verschil bepaalt XOR plus `trailing_zeros` op little-endian
x86-64 de **eerste** afwijkende byte in adresvolgorde. Het resultaat is het
verschil tussen unsigned bytes, geen numerieke vergelijking van gehele woorden.
De resterende nul tot zeven bytes worden afzonderlijk gelezen. Ook deze routine
laat DF=0 achter. Er worden geen SIMD-instructies of CPU-extensies toegevoegd.

`memset` en de overlapveilige `memmove` zijn ongewijzigd. Dit verruimt hun
contract niet; met name de bestaande nul-lengteroute van `memmove` wist DF niet.
Er is geen MMIO-, gedeeld-geheugen- of atomiciteitsgarantie toegevoegd.

## Verificatie en bewijsgrens

`tools/tests/test_kernel_memory.py` compileert de **werkelijke** kernelbron met
Rust1.90/O3 en voert vijf Linux/x86-64-tests uit. Geslaagd: alle64 combinaties
van bron-/bestemmingsalignment0..7 bij lengtes0..1025, oorspronkelijke pointer,
canaries, read-only bron, daadwerkelijk ontoegankelijke guard pages en exact
liggende bereikseinden. Lexicale tests variëren iedere eerste afwijkende
bytepositie, unsigned80/7f en00/ff, met een tegengesteld volgend verschil.
Een enkele asm-trampoline houdt STD→CALL→flagsmeting→CLD bij elkaar; Rust
hervat uitsluitend met DF=0. Overlappende memmove-bereiken worden apart getoetst.

Een tweede agent las de daadwerkelijke diff en controleerde Rust1.90/O3-
assembly/IR: juiste clobbers, correcte stackalignment bij de testtrampoline,
geen live redzonestorage en geen terugverwijzing van de lexicale testoracle
naar de geteste memcmp. Geen concrete blocker gevonden; dit is geen menselijke
audit of formeel bewijs.

Native releasebuild/ELF-inspectie en de gerichte scalar-instructiecontrole worden
bij de commit opgeslagen in `build/word-memory/logs/`. Echte M4-, netwerk- en
venstergates volgen op de geïntegreerde bron. Er wordt hier geen native
snelheidswinst geclaimd.

Een afzonderlijke, identiek geïnstrumenteerde scalar-QEMU-proef van snapshot,
exactEq en verplicht wissen leverde **geen** geldige meting: de baseline stopte
vóór M2:PASS. De rode rapporten en gegenereerde bronnen blijven onder
`build/word-memory/benchmark/e8b28780e470459e89b5f976eec03fd3/` bewaard. Een
begrensde diagnostic-herhaling verwierp al vóór CR3-wissel de ontbrekende
code-leaf; geen van beide uitkomsten is een geslaagde benchmark. De experimentele
driver staat buiten de productiebron, onder `build/word-memory/`. De relevante
vervolgmeting is de volledige echte venstersessie, met dezelfde pixelorakels en
zonder verruimde beveiligingscontroles.

Terugname is een gerichte revert van deze twee intrinsics. De snapshot- en
foutatomiciteitscontroles zijn bij deze stap niet aangepast.
